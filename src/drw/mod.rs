extern crate x11;
extern crate libc;

use std::ptr;

use x11::{ xlib, xft };

pub mod clrscheme;
pub mod fnt;

use self::clrscheme::ClrScheme;
use self::fnt::Fnt;

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

pub struct Drw {
    pub w: u32,
    pub h: u32,
    dpy: *mut xlib::Display,
    pub screen: i32,
    root: xlib::Window,
    drawable: xlib::Drawable,
    gc: xlib::GC,
    scheme: *mut ClrScheme,
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
            }
        }
    }

    // Draws a rectangle.
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, filled: bool, empty: bool, invert: bool) {
        let s = self.scheme;
        if !s.is_null() {
            if invert {
                unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).bg.pix) };
            } else {
                unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).fg.pix) };
            }

            if filled {
                unsafe { xlib::XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w + 1, h + 1) };
            } else if empty {
                unsafe { xlib::XDrawRectangle(self.dpy, self.drawable, self.gc, x, y, w, h) };
            }
        }
    }

    // Draws text. TODO c'est pas encore fini
    pub fn text(&mut self, x: i32, y: i32, mut w:u32, h:u32, text: &str, invert: bool) -> i32 {
        let s = self.scheme;
        let mut d = ptr::null_mut();
        if !s.is_null() {
            if self.fonts.len() > 0 {
                let render = x!= 0 || y != 0 || w != 0 || h != 0;   
                if !render {
                    w = !w;
                } else {
                    if invert {
                        unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).fg.pix) };
                    } else {
                        unsafe { xlib::XSetForeground(self.dpy, self.gc, (*s).bg.pix) };
                    }
                    unsafe { xlib::XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h) };
                    d = unsafe { xft:: XftDrawCreate(self.dpy, self.drawable, xlib::XDefaultVisual(self.dpy, self.screen), xlib::XDefaultColormap(self.dpy, self.screen)) };
                }

                let curfont = &self.fonts[0];
                let nextfont = &self.fonts[0];
                let mut charexists = false;
                loop {
                    let utf8str = text.as_bytes();
                    
                    if render {
                        let th = curfont.ascent + curfont.descent;
                        let ty = y + (h / 2) as i32 - (th / 2) + curfont.ascent;
                        let tx = x + (h / 2) as i32;
                        if invert {
                            unsafe { xft::XftDrawStringUtf8(d, &(*s).bg.rgb, curfont.xfont, tx, ty, utf8str.as_ptr(), utf8str.len() as i32) };
                        } else {
                            unsafe { xft::XftDrawStringUtf8(d, &(*s).fg.rgb, curfont.xfont, tx, ty, utf8str.as_ptr(), utf8str.len() as i32) };
                        } 
                    }
                    
                    if !charexists || nextfont != curfont {
                        break;
                    } else {
                        charexists = false;
                    }
                }
            }
        }
        if !d.is_null() {
            unsafe { xft::XftDrawDestroy(d) };
        }
        x
    }
    
    // Draws from a window.
    pub fn map(&mut self, win: xlib::Window, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            xlib::XCopyArea(self.dpy, self.drawable, win, self.gc, x, y, w, h, x, y);
            xlib::XSync(self.dpy, 0);
        }
    }
}
