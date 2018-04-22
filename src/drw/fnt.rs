extern crate x11;

use std::process;
use std::ffi::CString;

use x11::{ xlib, xft };

use drw::Drw;

/// Font class
pub struct Fnt {
    pub dpy: *mut xlib::Display,
    pub ascent: i32,
    pub descent: i32,
    pub h: u32,
    pub xfont: *mut xft::XftFont,
    pub pattern: *mut xft::FcPattern 
}

impl Fnt {
    pub fn new(drw: &mut Drw, fontname: Option<&str>, fontpattern: Option<xft::FcPattern>) -> Option<Fnt> {
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
            eprintln!("no font specified\n");
            process::exit(1);
        }
    }

    pub fn free(&self) {
        unsafe { xft::XftFontClose(self.dpy, self.xfont) };
    }

    /*fn getexts(&mut self, text: Vec<u8>, len: usize, tex: &xft::Extnts) {

    }*/
}

impl PartialEq for Fnt {
    fn eq(&self, other: &Fnt) -> bool {
        self.xfont == other.xfont
    }
}
