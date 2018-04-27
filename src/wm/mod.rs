extern crate x11;

use std::ffi::CString;

use x11::xlib;
use x11::xinerama;
use x11::keysym;

/// Monitor module
pub mod monitor;
/// Client module
pub mod client;

use drw::{ Drw, Cur };
use drw::clrscheme::{ Clr, ClrScheme };
use config;
use { CURNORMAL, isuniquegeom };
use self::monitor::Monitor;

/**
 * Stores the state of the Window Manager
 */
pub struct WM<'a> {
    pub drw: Drw<'a>,
    pub screen: i32,
    pub root: u64,
    pub running: bool,
    pub wmatom: Vec<xlib::Atom>,
    pub netatom: Vec<xlib::Atom>,
    pub cursor: Vec<Cur>,
    pub scheme: Vec<ClrScheme>,
    pub mons: Vec<Monitor<'a>>,
    pub selmonindex: usize,
    pub sw: u32, sh: u32,
    pub bh: u32,
    pub stext: String,
    pub numlockmask: u32
}

impl<'a> WM<'a> {
    /**
     * Constructor (inits the WM)
     */
    pub fn new(drw: Drw<'a>, screen: i32, root: u64, sw: u32, sh: u32) -> WM<'a> {
        let mut wm = WM {
            drw,
            screen,
            root,
            running: true,
            wmatom: Vec::new(),
            netatom: Vec::new(),
            cursor: Vec::new(),
            scheme: Vec::new(),
            mons: Vec::new(), 
            selmonindex: 0,
            sw, sh,
            bh: 0, 
            stext: String::from("dwm-rust"),
            numlockmask: 0
        }; 
        wm.bh = wm.drw.fonts[0].h + 2; 
        unsafe {
            // Init atoms
            wm.wmatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("WM_PROTOCOLS").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("WM_DELETE_WINDOW").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("WM_STATE").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("WM_TAKE_FOCUS").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy,CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_SUPPORTED").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_WM_NAME").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_WM_STATE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_WM_WINDOWN_TYPE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_WM_WINDOW_TYPE_DIALOG").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.drw.dpy, CString::new("_NET_CLIENT_LIST").unwrap().as_ptr(), 0));
            // Init cursors
            wm.cursor.push(Cur::new(&mut (wm.drw), 68)); // Normal
            wm.cursor.push(Cur::new(&mut (wm.drw), 120)); // Resize
            wm.cursor.push(Cur::new(&mut (wm.drw), 52)); // Move
            // Init color schemes
            wm.scheme.push(ClrScheme::new(
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::normfgcolor),
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::normbgcolor),
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::normbordercolor))); // Normal
            wm.scheme.push(ClrScheme::new(
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::selfgcolor),
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::selbgcolor),
                    Clr::new(wm.drw.dpy, wm.drw.screen, config::selbordercolor))); // Selected
            } 
        wm 
    } 

    /**
     * Updates the geometry
     */
    pub fn updategeom(&mut self) -> bool {
        let mut dirty = false;
        if unsafe { xinerama::XineramaIsActive(self.drw.dpy) != 0 } {
            let n = self.mons.len();
            let mut nn: i32 = 0;
            let mut unique = Vec::new();
            let info = unsafe { xinerama::XineramaQueryScreens(self.drw.dpy, &mut nn) };
            let info = unsafe { Vec::from_raw_parts(info, nn as usize, nn as usize) };


            for _ in 0..nn {
                unique.push(xinerama::XineramaScreenInfo { // Dummy value
                    height: 0, width: 0, screen_number: 0, x_org: 0, y_org: 0
                });
            }
            let j = 0;
            for i in 0..nn {
                if isuniquegeom(&unique, j, &info[i as usize]) {
                    j+1;
                    unique[j] = info[i as usize].clone();
                }
            }
            // xlib::XFree(info); // TODO
            nn = unique.len() as i32;
            if n <= nn as usize { // More physical monitors than Monitor in the wm : lets create new Monitors !
                for _ in 0..(nn-n as i32) {
                    self.mons.push(Monitor::new());
                }
                for i in n..unique.len().min(self.mons.len()) { // And lets update their data
                    if unique[i].x_org as i32 != self.mons[i].wx
                    || unique[i].y_org as i32 != self.mons[i].wy
                    || unique[i].width as u32 != self.mons[i].mw
                    || unique[i].height as u32 != self.mons[i].mh {
                        dirty = true;
                        self.mons[i].num = i as i32;
                        self.mons[i].mx = unique[i].x_org as i32;
                        self.mons[i].wx = unique[i].x_org as i32;
                        self.mons[i].my = unique[i].y_org as i32;
                        self.mons[i].wy = unique[i].y_org as i32;
                        self.mons[i].mw = unique[i].width as u32;
                        self.mons[i].ww = unique[i].width as u32;
                        self.mons[i].mh = unique[i].height as u32;
                        self.mons[i].wh = unique[i].height as u32;
                        self.mons[i].updatebarpos(self.bh);
                    }
                }
            } else { // Else, we're going to have to destroy monitors :(

            }
        } else {
            if self.mons.is_empty() {
                self.mons.push(Monitor::new());
            } 
            let m = &mut self.mons[0];
                if m.ww != self.sw {
                    dirty = true;
                    m.mw = self.sw; m.ww = self.sw;
                    m.mh = self.sh; m.wh = self.sh;
                    m.updatebarpos(self.bh);
                }
        }
        if dirty {
            // TODO
            self.selmonindex = 0;
            //self.selmon = Some(wintomon(wm, wm.root));
        }
        dirty
    }

    /**
     * Updates the status bars
     */
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
                    xlib::XCreateWindow(self.drw.dpy,
                                        self.root,
                                        m.wx, m.by, m.ww as u32, 
                                        self.bh, 
                                        0, 
                                        xlib::XDefaultDepth(self.drw.dpy, self.screen), 
                                        xlib::CopyFromParent as u32, 
                                        xlib::XDefaultVisual(self.drw.dpy, self.screen), 
                                        xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask, &mut wa) };
                unsafe { xlib::XDefineCursor(self.drw.dpy, m.barwin, self.cursor[CURNORMAL].cursor) };
                unsafe { xlib::XMapRaised(self.drw.dpy, m.barwin) };
            }
        }
    }

    /**
     * Updates the status bar text
     */
    pub fn updatestatus(&mut self) {
        // if(...) TODO
        let selmon = &mut self.mons[self.selmonindex];
        selmon.drawbar(&mut (self.drw), self.bh, &mut self.scheme, selmon, &self.stext[..]);
    }

    /**
     * Loads and grabs the keys defined in config::keys
     */
    pub fn grabkeys(&mut self) {
        self.updatenumlockmask();
        let modifiers = vec![0, xlib::LockMask, self.numlockmask, self.numlockmask|xlib::LockMask];

        unsafe { xlib::XUngrabKey(self.drw.dpy, xlib::AnyKey, xlib::AnyModifier, self.root) };
        for i in 0..config::keys.len() {
            let code = unsafe { xlib::XKeysymToKeycode(self.drw.dpy, config::keys[i].keysym) };
            if code != 0 {
                for j in 0..modifiers.len() {
                    unsafe { xlib::XGrabKey(self.drw.dpy, code as i32, config::keys[i].modif | modifiers[j], self.root, 1, xlib::GrabModeAsync, xlib::GrabModeAsync) };
                }
            }
        }
    }

    fn updatenumlockmask(&mut self) {
        let modmap = unsafe { (*xlib::XGetModifierMapping(self.drw.dpy)) };
        self.numlockmask = 0;
        let modifiermap = unsafe { Vec::from_raw_parts(modmap.modifiermap, 8 * modmap.max_keypermod as usize, 8 * modmap.max_keypermod as usize) };
        for i in 0..8 {
            for j in 0..modmap.max_keypermod {
                if modifiermap[(i * modmap.max_keypermod + j) as usize] == unsafe { xlib::XKeysymToKeycode(self.drw.dpy, keysym::XK_Num_Lock as u64) } {
                    self.numlockmask = 1 << i;
                }
            }
        }
        // unsafe { xlib::XFreeModifiermap(&mut modmap); } Causes a crash for some reason
    }
}


