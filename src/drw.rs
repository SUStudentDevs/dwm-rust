extern crate x11;
extern crate libc;

use std::ptr;
use std::process;
use std::ffi::CString;

use x11::xlib;
use x11::xft;
use x11::xrender;

/// Stores a color
pub struct Clr {
    pix: u64,
    rgb: xft::XftColor
}

impl Clr {
    pub fn new(dpy: &mut xlib::Display, screen: i32, clrname: &str) -> Clr {
        let mut rgb = xft::XftColor {
            pixel: 0,
            color: xrender::XRenderColor { red: 0, green: 0, blue: 0, alpha: 0 }
        };
        if unsafe { xft::XftColorAllocName(dpy,  //Y'a un pb ici
                                   xlib::XDefaultVisual(dpy, screen),
                                   xlib::XDefaultColormap(dpy, screen),
                                   CString::new(clrname).unwrap().as_ptr(),
                                   &mut rgb) } == 0 {
            eprintln!("Error, cannot allocate color {:?}\n", clrname);
            process::exit(1)
        }
        Clr {
            pix: rgb.pixel,
            rgb: rgb
        }
    }
}

/// Stores a color scheme
pub struct ClrScheme {
    fg: Clr,
    bg: Clr,
    border: Clr
}

impl ClrScheme {
    pub fn new(fg: Clr, bg: Clr, border: Clr) -> ClrScheme {
        ClrScheme {
            fg,
            bg,
            border
        }
    }
}

/// Stores a cursor
pub struct Cur {
    pub cursor: xlib::Cursor
}

impl Cur {
    pub fn new(drw: &mut Drw, shape: u32) -> Cur {
        Cur {
            cursor: unsafe { xlib::XCreateFontCursor(drw.dpy, shape) }
        } 
    }
}

/// Stores a font
pub struct Fnt {
    dpy: *mut xlib::Display,
    pub ascent: i32,
    pub descent: i32,
    pub h: u32,
    xfont: *mut xft::XftFont,
    pattern: *mut xft::FcPattern 
}

impl Fnt {
    fn new(drw: &mut Drw, fontname: Option<&str>, fontpattern: Option<xft::FcPattern>) -> Option<Fnt> {
        if let Some(ftn) = fontname {
            let ftn_c = CString::new(ftn).unwrap().as_ptr();
            let xfont = unsafe { xft::XftFontOpenName((*drw).dpy, (*drw).screen, ftn_c) };
            if xfont.is_null() {
                eprintln!("error, cannot load font: {:?}\n", fontname);
                None
            } else {
                let pattern = unsafe { xft::XftNameParse(ftn_c) };
                if pattern.is_null() {
                    eprintln!("error, cannot load font: {:?}\n", fontname);
                    None
                } else {
                    unsafe { 
                        Some(Fnt {
                            dpy: (*drw).dpy,
                            ascent: (*xfont).ascent,
                            descent: (*xfont).descent,
                            h: ((*xfont).ascent + (*xfont).descent) as u32,
                            xfont: xfont,
                            pattern: pattern
                        }) 
                    }
                }
            }
        } else if let Some(mut ftp) = fontpattern {
            let xfont = unsafe { xft::XftFontOpenPattern((*drw).dpy, &mut ftp as *mut xft::FcPattern) };
            if !xfont.is_null() {
                eprintln!("error, cannot load font pattern\n");
                None
            } else {
                unsafe {
                    Some(Fnt {
                        dpy: (*drw).dpy,
                        ascent: (*xfont).ascent,
                        descent: (*xfont).descent,
                        h: ((*xfont).ascent + (*xfont).descent) as u32,
                        xfont: xfont,
                        pattern: &mut ftp as *mut xft::FcPattern
                    })
                }
            }
        } else {
            println!("no font specified\n");
            process::exit(1);
        }
    }

    fn free(&self) {
        unsafe { xft::XftFontClose(self.dpy, self.xfont) };
    }
}

pub struct Drw {
    pub w: u32,
    pub h: u32,
    dpy: *mut xlib::Display,
    pub screen: i32,
    root: xlib::Window,
    drawable: xlib::Drawable,
    gc: xlib::GC,
    scheme: *mut ClrScheme,
    pub fontcount: usize,
    pub fonts: Vec<Fnt>
}

impl Drw {
    pub fn new(dpy: &mut xlib::Display, screen: i32, root: xlib::Window, w: u32, h:u32) -> Drw {
        let drw = Drw {
            dpy,
            screen,
            root,
            w,
            h,
            drawable: unsafe { xlib::XCreatePixmap(dpy, root, w, h, xlib::XDefaultDepth(dpy, screen) as u32) },
            gc: unsafe { xlib::XCreateGC(dpy, root, 0, ptr::null_mut()) },
            fontcount: 0,
            fonts: Vec::new(),
            scheme: ptr::null_mut()
        };
        drw
    }

    pub fn free(&mut self) {
        for f in &self.fonts {
            f.free();
        }
        unsafe {
            xlib::XFreePixmap(self.dpy, self.drawable);
            xlib::XFreeGC(self.dpy, self.gc);
        }
    }

    pub fn setscheme(&mut self, scheme: &mut ClrScheme) {
        self.scheme = scheme;
    }

    pub fn load_fonts(&mut self, fontnames: Vec<&str>) {
        for f in fontnames {
            if let Some(font) = Fnt::new(self, Some(f), None) {
                self.fonts.push(font);
                self.fontcount = self.fontcount + 1;
            }
        }
    }

    // Draws a rectangle.
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, filled: bool, _: bool, invert: bool) {
        let s = self.scheme;
        if !s.is_null() {
            if invert {
                unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).bg.pix) };
            } else {
                unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).fg.pix) };
            }

            if filled {
                unsafe { xlib::XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w + 1, h + 1) };
            } else {
                unsafe { xlib::XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h) };
            }
        }
    }

    // Draws text.
    pub fn text(&mut self, x: i32, y: i32, mut w:u32, h:u32, text: &str, invert: bool) -> i32 {
        let s = self.scheme;
        if !s.is_null() {
            if self.fontcount > 0 {
                let render = x;   
                if !(x>0 || y>0 || w>0 || h>0) {
                    w = !w;
                } else {
                    if invert {
                        unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).bg.pix) };
                    } else {
                        unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).fg.pix) };
                    }
                    unsafe { xlib::XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h) };
                    unsafe { xlib::XDefaultVisual(self.dpy, self.screen) };
                }

                let curfont = &self.fonts[0];
                /*loop {
                    let utf8strlen = 0;
                    let utf8str = text;
                    // TODO : finir ça
                }*/
            }
        }
        0
    }
    
    // Draws from a window.
    pub fn map(&mut self, win: xlib::Window, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            xlib::XCopyArea(self.dpy, self.drawable, win, self.gc, x, y, w, h, x, y);
            xlib::XSync(self.dpy, 0);
        }
    }
}
