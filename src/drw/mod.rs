extern crate x11;
extern crate libc;

use std::ptr;

use x11::{ xlib, xft };

pub mod clrscheme;
pub mod fnt;

use self::clrscheme::ClrScheme;
use self::fnt::Fnt;

/**
 * Stores a cursor (wrapper around xlib::Cursor)
 */
pub struct Cur {
    pub cursor: xlib::Cursor
}

impl Cur {
    /**
     * Constructor
     */
    pub fn new(drw: &mut Drw, shape: u32) -> Cur {
        Cur {
            cursor: unsafe { xlib::XCreateFontCursor(drw.dpy, shape) }
        } 
    }
}

/**
 * Stores a drawble area (related to a Display)
 */
pub struct Drw<'a> {
    pub w: u32,
    pub h: u32,
    pub dpy: &'a mut xlib::Display,
    pub screen: i32,
    root: xlib::Window,
    drawable: xlib::Drawable,
    gc: xlib::GC,
    scheme: *mut ClrScheme,
    pub fonts: Vec<Fnt>
}

impl<'a> Drw<'a> {
    /**
     * Constructor
     */
    pub fn new(dpy: &mut xlib::Display, screen: i32, root: xlib::Window, w: u32, h:u32) -> Drw {
        let mut drw = Drw {
            dpy,
            screen,
            root,
            w,
            h,
            drawable: 0,
            gc: ptr::null_mut(),
            fonts: Vec::new(),
            scheme: ptr::null_mut()
        };
        drw.drawable = unsafe { xlib::XCreatePixmap(drw.dpy, root, w, h, xlib::XDefaultDepth(drw.dpy, screen) as u32) };
        drw.gc = unsafe { xlib::XCreateGC(drw.dpy, root, 0, ptr::null_mut()) };
        drw
    }

    /**
     * Destructor
     */
    pub fn free(&mut self) {
        for f in &mut self.fonts {
            f.free();
        }
        unsafe {
            xlib::XFreePixmap(self.dpy, self.drawable);
            xlib::XFreeGC(self.dpy, self.gc);
        }
    }

    /**
     * Changes the color scheme
     */
    pub fn setscheme(&mut self, scheme: &mut ClrScheme) {
        self.scheme = scheme;
    }

    /**
     * Loads fonts
     */
    pub fn load_fonts(&mut self, fontnames: Vec<&str>) {
        for f in fontnames {
            if let Some(font) = Fnt::new(self, Some(f), None) {
                self.fonts.push(font);
            }
        }
    }

    /**
     * Draws a rectangle
     */
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

    /**
     * Draws text, and returns text width
     */
    pub fn text(&mut self, mut x: i32, y: i32, mut w:u32, h:u32, text: &str, invert: bool) -> i32 {
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

                let curfont = &mut self.fonts[0];
                let mut charexists = false;
                let mut tex = fnt::Extnts { // Dummy value
                    w: 0, h: 0
                };
                loop {
                    let utf8str = text.as_bytes();
                    curfont.getexts(utf8str.to_vec(), &mut tex);
                    
                    if render {
                        let th = curfont.ascent + curfont.descent;
                        let ty = y + (h / 2) as i32 - (th / 2) + curfont.ascent;
                        let tx = x + (h / 2) as i32;
                        println!("font : {:?}", unsafe { *(curfont.xfont) });
                        if invert {
                            unsafe { xft::XftDrawStringUtf8(d, &(*s).bg.rgb, curfont.xfont, tx, ty, utf8str.as_ptr(), utf8str.len() as i32) };
                        } else {
                            unsafe { xft::XftDrawStringUtf8(d, &(*s).fg.rgb, curfont.xfont, tx, ty, utf8str.as_ptr(), utf8str.len() as i32) };
                        } 
                    }
                    x += tex.w as i32;
                    w -= tex.w;

                    if !charexists /* || nextfont != curfont*/ {
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
    
    /**
     * Draws content from a Window on the screen
     */
    pub fn map(&mut self, win: xlib::Window, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            xlib::XCopyArea(self.dpy, self.drawable, win, self.gc, x, y, w, h, x, y);
            xlib::XSync(self.dpy, 0);
        }
    }
}
