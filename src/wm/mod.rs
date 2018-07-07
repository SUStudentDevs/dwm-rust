extern crate x11;

use std::ffi::CString;

use x11::xlib;
use x11::xinerama;
use x11::keysym;

/// Monitor module
pub mod monitor;
/// Client module
pub mod client;

use { CURNORMAL, SCHEMENORM, isuniquegeom };
use self::monitor::Monitor;
use drw;
use drw::{ Drw, Cur };
use drw::clrscheme;
use drw::clrscheme::{ Clr, ClrScheme };
use config;

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

/**
 * Inits the window manager
 */
pub fn initWm(drw: Drw, screen: i32, root: u64, sw: u32, sh: u32) -> WM {
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
        wm.cursor.push(drw::createCur(&mut (wm.drw), 68)); // Normal
        wm.cursor.push(drw::createCur(&mut (wm.drw), 120)); // Resize
        wm.cursor.push(drw::createCur(&mut (wm.drw), 52)); // Move
        // Init color schemes
        wm.scheme.push(clrscheme::createClrScheme(
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::normfgcolor),
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::normbgcolor),
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::normbordercolor))); // Normal
        wm.scheme.push(clrscheme::createClrScheme(
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::selfgcolor),
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::selbgcolor),
            clrscheme::createClr(wm.drw.dpy, wm.drw.screen, config::selbordercolor))); // Selected
    }
    wm
}

// /**
//  * Updates the geometry
//  */
// pub fn updategeom(&mut self) -> bool {
//     let mut dirty = false;
//     if unsafe { xinerama::XineramaIsActive(self.drw.dpy) != 0 } {
//         let n = self.mons.len();
//         let mut nn: i32 = 0;
//         let mut unique = Vec::new();
//         let info = unsafe { xinerama::XineramaQueryScreens(self.drw.dpy, &mut nn) };
//         let info = unsafe { Vec::from_raw_parts(info, nn as usize, nn as usize) };


//         for _ in 0..nn {
//             unique.push(xinerama::XineramaScreenInfo { // Dummy value
//                 height: 0, width: 0, screen_number: 0, x_org: 0, y_org: 0
//             });
//         }
//         let j = 0;
//         for i in 0..nn {
//             if isuniquegeom(&unique, j, &info[i as usize]) {
//                 j+1;
//                 unique[j] = info[i as usize].clone();
//             }
//         }
//         // xlib::XFree(info); // TODO
//         nn = unique.len() as i32;
//         if n <= nn as usize { // More physical monitors than Monitor in the wm : lets create new Monitors !
//             for _ in 0..(nn-n as i32) {
//                 self.mons.push(Monitor::new());
//             }
//             for i in n..unique.len().min(self.mons.len()) { // And lets update their data
//                 if unique[i].x_org as i32 != self.mons[i].wx
//                     || unique[i].y_org as i32 != self.mons[i].wy
//                     || unique[i].width as u32 != self.mons[i].mw
//                     || unique[i].height as u32 != self.mons[i].mh {
//                         dirty = true;
//                         self.mons[i].num = i as i32;
//                         self.mons[i].mx = unique[i].x_org as i32;
//                         self.mons[i].wx = unique[i].x_org as i32;
//                         self.mons[i].my = unique[i].y_org as i32;
//                         self.mons[i].wy = unique[i].y_org as i32;
//                         self.mons[i].mw = unique[i].width as u32;
//                         self.mons[i].ww = unique[i].width as u32;
//                         self.mons[i].mh = unique[i].height as u32;
//                         self.mons[i].wh = unique[i].height as u32;
//                         self.mons[i].updatebarpos(self.bh);
//                     }
//             }
//         } else { // Else, we're going to have to destroy monitors :(

//         }
//     } else {
//         if self.mons.is_empty() {
//             self.mons.push(Monitor::new());
//         }
//         let m = &mut self.mons[0];
//         if m.ww != self.sw {
//             dirty = true;
//             m.mw = self.sw; m.ww = self.sw;
//             m.mh = self.sh; m.wh = self.sh;
//             m.updatebarpos(self.bh);
//         }
//     }
//     if dirty {
//         // TODO
//         self.selmonindex = 0;
//     }
//     dirty
// }

// /**
//  * Updates the status bars
//  */
// pub fn updatebars(&mut self) {
//     let mut wa = xlib::XSetWindowAttributes {
//         background_pixmap: xlib::ParentRelative as u64,
//         background_pixel: 0,
//         border_pixmap: xlib::CopyFromParent as u64,
//         border_pixel: 0,
//         bit_gravity: xlib::ForgetGravity,
//         win_gravity: xlib::NorthWestGravity,
//         backing_store: xlib::NotUseful,
//         backing_planes: u64::max_value(),
//         backing_pixel: 0,
//         save_under: 0,
//         event_mask: xlib::ButtonPressMask|xlib::ExposureMask,
//         do_not_propagate_mask: 0,
//         override_redirect: 1,
//         colormap: xlib::CopyFromParent as u64,
//         cursor: self.cursor[CURNORMAL].cursor
//     };
//     for mut m in self.mons.iter_mut() {
//         if m.barwin == 0 {
//             m.barwin = unsafe {
//                 xlib::XCreateWindow(self.drw.dpy,
//                                     self.root,
//                                     m.wx, m.by, m.ww as u32,
//                                     self.bh,
//                                     0,
//                                     xlib::XDefaultDepth(self.drw.dpy, self.screen),
//                                     xlib::CopyFromParent as u32,
//                                     xlib::XDefaultVisual(self.drw.dpy, self.screen),
//                                     xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask, &mut wa) };
//             unsafe { xlib::XDefineCursor(self.drw.dpy, m.barwin, self.cursor[CURNORMAL].cursor) };
//             unsafe { xlib::XMapRaised(self.drw.dpy, m.barwin) };
//         }
//     }
// }

// /**
//  * Updates the status bar text
//  */
// pub fn updatestatus(&mut self) {
//     // if(...) TODO
//     let selmon = &mut self.mons[self.selmonindex];
//     selmon.drawbar(&mut (self.drw), self.bh, &mut self.scheme, selmon, &self.stext[..]);
// }

// /**
//  * Loads and grabs the keys defined in config::keys
//  */
// pub fn grabkeys(&mut self) {
//     self.updatenumlockmask();
//     let modifiers = vec![0, xlib::LockMask, self.numlockmask, self.numlockmask|xlib::LockMask];

//     unsafe { xlib::XUngrabKey(self.drw.dpy, xlib::AnyKey, xlib::AnyModifier, self.root) };
//     for i in 0..config::keys.len() {
//         let code = unsafe { xlib::XKeysymToKeycode(self.drw.dpy, config::keys[i].keysym) };
//         if code != 0 {
//             for j in 0..modifiers.len() {
//                 unsafe { xlib::XGrabKey(self.drw.dpy, code as i32, config::keys[i].modif | modifiers[j], self.root, 1, xlib::GrabModeAsync, xlib::GrabModeAsync) };
//             }
//         }
//     }
// }

// /**
//  * Grabs buttons
//  */
// pub fn grabbuttons(&mut self, c: &Client, focused: bool) {
//     self.updatenumlockmask();
//     let modifiers = vec![0, xlib::LockMask, self.numlockmask, xlib::LockMask|self.numlockmask];
//     unsafe { xlib::XUngrabButton(self.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, c.win) };
//     if focused {
//         for b in config::buttons.iter() {
//             /*if b.click == ClkClientWin {
//             TODO
//         }*/
//         }
//     } else {
//         unsafe { xlib::XGrabButton(self.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, c.win, 0, (xlib::ButtonPressMask|xlib::ButtonReleaseMask) as u32, xlib::GrabModeAsync, xlib::GrabModeSync, 0, 0) };
//     }
// }

// fn updatenumlockmask(&mut self) {
//     let modmap = unsafe { (*xlib::XGetModifierMapping(self.drw.dpy)) };
//     self.numlockmask = 0;
//     let modifiermap = unsafe { Vec::from_raw_parts(modmap.modifiermap, 8 * modmap.max_keypermod as usize, 8 * modmap.max_keypermod as usize) };
//     for i in 0..8 {
//         for j in 0..modmap.max_keypermod {
//             if modifiermap[(i * modmap.max_keypermod + j) as usize] == unsafe { xlib::XKeysymToKeycode(self.drw.dpy, keysym::XK_Num_Lock as u64) } {
//                 self.numlockmask = 1 << i;
//             }
//         }
//     }
//     // unsafe { xlib::XFreeModifiermap(&mut modmap); } TODO Causes a crash for some reason
// }

// /**
//  * Manage a new Window
//  */
// pub fn manage(&mut self, w: xlib::Window, wa: &xlib::XWindowAttributes) {
//     let mut c = Client::new(w, wa, self.selmonindex);
//     c.updatetitle();
//     let mut trans = 0;
//     if unsafe { xlib::XGetTransientForHint(self.drw.dpy, w, &mut trans) } != 0 {
//         if let Some(t) = Client::from(trans, &self.mons) {
//             c.monindex = t.monindex;
//             c.tags = t.tags;
//         } else {
//             c.monindex = self.selmonindex;
//             c.applyrules();
//         }
//     } else {
//         c.monindex = self.selmonindex;
//         c.applyrules();
//     }
//     let mon = &self.mons[c.monindex];
//     if c.x + c.width() as i32 > mon.mx + mon.mw as i32 {
//         c.x = mon.mx + mon.mw as i32 - c.width() as i32;
//     }
//     if c.y + c.height() as i32 > mon.my + mon.mw as i32 {
//         c.y = mon.my + mon.mw as i32 - c.height() as i32;
//     }
//     c.x = c.x.max(mon.mx);
//     c.y = c.y.max(if mon.by == mon.my && c.x + (c.w/2) as i32 >= mon.wx && c.x + ((c.w/2) as i32) < mon.wx + mon.ww as i32 { self.bh as i32 } else { mon.my });
//     let mut wc = xlib::XWindowChanges {
//         x: 0, y: 0, width:0, height: 0, border_width: c.bw as i32, sibling: 0, stack_mode: 0
//     };
//     unsafe { xlib::XConfigureWindow(self.drw.dpy, w, xlib::CWBorderWidth as u32, &mut wc) };
//     unsafe { xlib::XSetWindowBorder(self.drw.dpy, w, self.scheme[SCHEMENORM].border.pix) };
//     c.configure(self.drw.dpy);
//     c.updatewindowtype(self.drw.dpy, &self.netatom);
//     c.updatesizehints(self.drw.dpy);
//     c.updatewmhints(self.drw.dpy, &self.mons[self.selmonindex]);
//     unsafe { xlib::XSelectInput(self.drw.dpy, w, xlib::EnterWindowMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::StructureNotifyMask) };
//     // self.grabbuttons(&c, false); TODO
//     if !c.isfloating {
//         c.isfloating = trans != 0 || c.isfixed;
//         c.oldstate = c.isfloating;
//     }
//     if c.isfloating {
//         unsafe { xlib::XRaiseWindow(self.drw.dpy, c.win) };
//     }
//     // TODO
//     unsafe { xlib::XMapWindow(self.drw.dpy, c.win) };
//     // focus(None) TODO
// }
