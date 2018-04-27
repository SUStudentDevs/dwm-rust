extern crate x11;

use std::process;
use std::ffi::CString;

use x11::{ xlib, xft, xrender };

use drw::Drw;

/**
 * Font extent (width and height)
 */
pub struct Extnts {
    pub w: u32,
    pub h: u32
}

/**
 * Stores a font (wrapper around xft::XftFont struct)
 */
pub struct Fnt {
    pub dpy: *mut xlib::Display,
    pub ascent: i32,
    pub descent: i32,
    pub h: u32,
    pub xfont: *mut xft::XftFont,
    pub pattern: *mut xft::FcPattern 
}

impl Fnt {
    /**
     * Constructor
     */
    pub fn new(drw: &mut Drw, fontname: Option<&str>, fontpattern: Option<xft::FcPattern>) -> Option<Fnt> {
        if let Some(ftn) = fontname {
            let ftn_c = CString::new(ftn).unwrap().as_ptr();
            let xfont = unsafe { xft::XftFontOpenName(drw.dpy, drw.screen, ftn_c) };
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
                            dpy: &mut *drw.dpy,
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
            let xfont = unsafe { xft::XftFontOpenPattern((*drw).dpy, &mut ftp) };
            if !xfont.is_null() {
                eprintln!("error, cannot load font pattern\n");
                None
            } else {
                unsafe {
                    Some(Fnt {
                        dpy: &mut *drw.dpy,
                        ascent: (*xfont).ascent,
                        descent: (*xfont).descent,
                        h: ((*xfont).ascent + (*xfont).descent) as u32,
                        xfont: xfont,
                        pattern: &mut ftp
                    })
                }
            }
        } else {
            eprintln!("no font specified\n");
            process::exit(1);
        }
    }

    /**
     * Destructor (frees xfont)
     */
    pub fn free(&mut self) {
        unsafe { xft::XftFontClose(self.dpy, self.xfont) };
    }

    pub fn getexts(&mut self, text: Vec<u8>, tex: &mut Extnts) {
        let mut ext = xrender::XGlyphInfo { // Dummy value
            height: 0, width: 0, x: 0, y: 0, xOff: 0, yOff: 0
        };
        unsafe { xft::XftTextExtentsUtf8(self.dpy, self.xfont, text.as_ptr(), text.len() as i32, &mut ext) }
        tex.h = self.h;
        tex.w = ext.xOff as u32;
    }

    pub fn getexts_width(&mut self, text: Vec<u8>) -> u32 {
        let mut tex = Extnts { // Dummy value
            w: 0, h: 0
        };
        self.getexts(text, &mut tex);
        tex.w
    }
}

impl PartialEq for Fnt {
    fn eq(&self, other: &Fnt) -> bool {
        self.xfont == other.xfont
    }
}
