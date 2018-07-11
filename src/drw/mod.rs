extern crate x11;
extern crate libc;

use std::ptr;

use x11::{ xlib, xft };

pub mod clrscheme;
pub mod fnt;

use self::clrscheme::ClrScheme;
use self::fnt::Fnt;

use SCHEMENORM;
use SCHEMESEL;

/**
 * Stores a cursor (wrapper around xlib::Cursor)
 */
pub struct Cur {
    pub cursor: xlib::Cursor
}

/**
 * Creates a new cursor for a drawable area
 */
pub fn createCur(drw: &mut Drw, shape: u32) -> Cur {
    Cur {
        cursor: unsafe { xlib::XCreateFontCursor(drw.dpy, shape)}
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
    scheme: *const ClrScheme,
    pub fonts: Vec<Fnt>
}

/**
 * Creates a drawable area for a display
 */
pub fn createDrw(dpy: &mut xlib::Display, screen: i32, root: xlib::Window, w: u32, h:u32) -> Drw {
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

// /**
//  * Destructor
//  */
// pub fn freeDrw(&mut self) {
//     for f in &mut self.fonts {
//         f.free(self.dpy);
//     }
//     unsafe {
//         xlib::XFreePixmap(self.dpy, self.drawable);
//         xlib::XFreeGC(self.dpy, self.gc);
//     }
// }

/**
 * Changes the color scheme
 */
pub fn setScheme<'a>(drw: Drw<'a>, scheme: &ClrScheme) -> Drw<'a> {
    Drw { scheme, ..drw }
}

/**
 * Loads fonts
 */
pub fn loadFonts<'a>(mut drw: Drw<'a>, fontnames: Vec<&str>) -> Drw<'a> {
    for f in fontnames {
        if let Some(font) = fnt::createFont(drw.dpy, drw.screen, Some(f), None) {
            drw.fonts.push(font);
        }
    }
    drw
}

/**
 * Draws a rectangle
 */
pub fn rect(drw: Drw, x: i32, y: i32, w: u32, h: u32, filled: bool, invert: bool) -> Drw {
    let s = drw.scheme;
    if !s.is_null() {
        if invert {
            unsafe { xlib::XSetForeground(drw.dpy, drw.gc, (*s).bg.pix) };
        } else {
            unsafe { xlib::XSetForeground(drw.dpy, drw.gc, (*s).fg.pix) };
        }

        if filled {
            unsafe { xlib::XFillRectangle(drw.dpy, drw.drawable, drw.gc, x, y, w + 1, h + 1) };
        } else {
            unsafe { xlib::XDrawRectangle(drw.dpy, drw.drawable, drw.gc, x, y, w, h) };
        }
    }
    drw
}

/**
 * Draws text, and returns text width
 */
pub fn text<'a>(drw: Drw<'a>, mut x: i32, y: i32, mut w:u32, h:u32, text: &str, invert: bool) -> (Drw<'a>, i32) {
    let s = drw.scheme;
    let mut d = ptr::null_mut();
    if !s.is_null() {
        if drw.fonts.len() > 0 {
            let render = x!= 0 || y != 0 || w != 0 || h != 0;
            if !render {
                w = !w;
            } else {
                if invert {
                    unsafe { xlib::XSetForeground(drw.dpy, drw.gc, (*s).fg.pix) };
                } else {
                    unsafe { xlib::XSetForeground(drw.dpy, drw.gc, (*s).bg.pix) };
                }
                unsafe { xlib::XFillRectangle(drw.dpy, drw.drawable, drw.gc, x, y, w, h) };
                d = unsafe { xft:: XftDrawCreate(drw.dpy, drw.drawable, xlib::XDefaultVisual(drw.dpy, drw.screen), xlib::XDefaultColormap(drw.dpy, drw.screen)) };
            }

            let curfont = &drw.fonts[0];
            let mut charexists = false;
            let mut tex = fnt::Extnts { // Dummy value
                w: 0, h: 0
            };
            loop {
                let utf8str = text.as_bytes();
                fnt::getexts(curfont, drw.dpy, utf8str.to_vec(), &mut tex);

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
    (drw, x)
}

/**
 * Width of a text
 */
pub fn textw<'a>(s: &str, drw: Drw<'a>) -> (Drw<'a>, u32) {
    let (drw, w) = text(drw, 0, 0, 0, 0, s, false);
    let h = drw.fonts[0].h;
    (drw, w as u32 + h)
}

/**
 * Draws content from a Window on the screen
 */
pub fn mapWindow(drw: Drw, win: xlib::Window, x: i32, y: i32, w: u32, h: u32) -> Drw {
    unsafe {
        xlib::XCopyArea(drw.dpy, drw.drawable, win, drw.gc, x, y, w, h, x, y);
        xlib::XSync(drw.dpy, 0);
    }
    drw
}
