extern crate x11;

use std::process;
use std::ffi::CString;

use x11::{ xlib, xft, xrender };

/// Color class
pub struct Clr {
    pub pix: u64,
    pub rgb: xft::XftColor
}

impl Clr {
    pub fn new(dpy: &mut xlib::Display, screen: i32, clrname: &str) -> Clr {
        let mut rgb = xft::XftColor {
            pixel: 0,
            color: xrender::XRenderColor { red: 0, green: 0, blue: 0, alpha: 0 }
        };
        if unsafe { xft::XftColorAllocName(dpy,
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

/// ColorScheme class
pub struct ClrScheme {
    pub fg: Clr,
    pub bg: Clr,
    pub border: Clr
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
