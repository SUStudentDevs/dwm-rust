extern crate x11;

use std::ptr;
use std::ffi::CString;
use std::collections::LinkedList;

use x11::xlib;
use x11::xinerama;

pub mod monitor;
pub mod client;

use drw::{ Drw, Cur };
use drw::clrscheme::{ Clr, ClrScheme };
use config;
use CURNORMAL;
use self::monitor::Monitor;

// Window Manager main class
pub struct WM<'a> {
    pub dpy: &'a mut xlib::Display,
    pub drw: Drw,
    pub screen: i32,
    pub root: u64,
    pub running: bool,
    pub wmatom: Vec<xlib::Atom>,
    pub netatom: Vec<xlib::Atom>,
    pub cursor: Vec<Cur>,
    pub scheme: Vec<ClrScheme>,
    pub mons: LinkedList<Monitor<'a>>,
    pub selmon: *mut Monitor<'a>,
    pub sw: u32, sh: u32,
    pub bh: u32,
    pub stext: String
}

impl<'a> WM<'a> {
    // Inits the WM
    pub fn new(dpy: &mut xlib::Display, drw: Drw, screen: i32, root: u64, sw: u32, sh: u32) -> WM {
        let mut wm = WM {
            dpy,
            drw,
            screen,
            root,
            running: true,
            wmatom: Vec::new(),
            netatom: Vec::new(),
            cursor: Vec::new(),
            scheme: Vec::new(),
            mons: LinkedList::new(), 
            selmon: ptr::null_mut(),
            sw, sh,
            bh: 0, 
            stext: String::from("dwm-rust")
        }; 
        wm.bh = wm.drw.fonts[0].h + 2; 
        unsafe {
            // Init atoms
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_PROTOCOLS").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_DELETE_WINDOW").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_STATE").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_TAKE_FOCUS").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy,CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_SUPPORTED").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_NAME").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_STATE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_WINDOWN_TYPE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_WINDOW_TYPE_DIALOG").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_CLIENT_LIST").unwrap().as_ptr(), 0));
            // Init cursors
            wm.cursor.push(Cur::new(&mut (wm.drw), 68)); // Normal
            wm.cursor.push(Cur::new(&mut (wm.drw), 120)); // Resize
            wm.cursor.push(Cur::new(&mut (wm.drw), 52)); // Move
            // Init color schemes
            wm.scheme.push(ClrScheme::new(
                    Clr::new(wm.dpy, wm.drw.screen, config::normfgcolor),
                    Clr::new(wm.dpy, wm.drw.screen, config::normbgcolor),
                    Clr::new(wm.dpy, wm.drw.screen, config::normbordercolor))); // Normal
            wm.scheme.push(ClrScheme::new(
                    Clr::new(wm.dpy, wm.drw.screen, config::selfgcolor),
                    Clr::new(wm.dpy, wm.drw.screen, config::selbgcolor),
                    Clr::new(wm.dpy, wm.drw.screen, config::selbordercolor))); // Selected
            } 
        wm 
    } 

    /// Updates the geometry
    pub fn updategeom(&mut self) -> bool {
        let mut dirty = false;
        if unsafe { xinerama::XineramaIsActive(self.dpy) }!=0 {
            // TODO
        } else {
            if self.mons.is_empty() {
                self.mons.push_front(Monitor::new());
            } 
            if let Some(mut m) = self.mons.front_mut() {
                if m.ww != self.sw {
                    dirty = true;
                    m.mw = self.sw; m.ww = self.sw;
                    m.mh = self.sh; m.wh = self.sh;
                    m.updatebarpos(self.bh);
                }
            }
        }
        if dirty {
            // TODO
            if let Some(mon) = self.mons.front_mut() {
                self.selmon = mon; // TODO
            }
            //wm.selmon = Some(wintomon(wm, wm.root));
        }
        dirty
    }

    /// Updates the status bars
    pub fn updatebars(&mut self) {
        let mut wa = xlib::XSetWindowAttributes {
            background_pixmap: xlib::ParentRelative as u64,
            background_pixel: 0, 
            border_pixmap: xlib::CopyFromParent as u64, 
            border_pixel: 0, 
            bit_gravity: xlib::ForgetGravity, 
            win_gravity: xlib::NorthWestGravity, 
            backing_store: xlib::NotUseful, 
            backing_planes: u64::max_value(), 
            backing_pixel: 0, 
            save_under: 0,
            event_mask: xlib::ButtonPressMask|xlib::ExposureMask,
            do_not_propagate_mask: 0,
            override_redirect: 1,
            colormap: xlib::CopyFromParent as u64, 
            cursor: self.cursor[CURNORMAL].cursor
        };
        for mut m in &mut self.mons {
            if m.barwin == 0 {
                m.barwin = unsafe { 
                    xlib::XCreateWindow(self.dpy,
                                        self.root,
                                        m.wx, m.by, m.ww as u32, 
                                        self.bh, 
                                        0, 
                                        xlib::XDefaultDepth(self.dpy, self.screen), 
                                        xlib::CopyFromParent as u32, 
                                        xlib::XDefaultVisual(self.dpy, self.screen), 
                                        xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask, &mut wa) };
                unsafe { xlib::XDefineCursor(self.dpy, m.barwin, self.cursor[CURNORMAL].cursor) };
                unsafe { xlib::XMapRaised(self.dpy, m.barwin) };
            }
        }
    }

    // Updates the status text
    pub fn updatestatus(&mut self) {
        // if(...) TODO
        unsafe { &(*self.selmon).drawbar(&mut (self.drw), self.bh, &mut self.scheme, &mut (*self.selmon), &self.stext[..]) };
    }
}


