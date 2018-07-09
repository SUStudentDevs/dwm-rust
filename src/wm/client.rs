extern crate x11;

use std::ptr;
use std::mem::size_of;

use x11::xlib;

use { NETWMSTATE, NETWMWINDOWTYPE, NETWMFULLSCREEN, NETWMWINDOWTYPEDIALOG };
use wm::WM;
use wm::workspace::Workspace;
use config;

/**
 * Stores a Client (wrapper around the xlib::Window struct)
 */
pub struct Client<'a> {
    pub name: &'a str,
    pub mina: f32, pub maxa: f32,
    pub x: i32, pub y: i32, pub w: u32, pub h: u32,
    pub bw: u32,
    pub tags: u32,
    pub isfixed: bool, pub isfloating: bool, pub isurgent: bool, pub neverfocus: bool, pub oldwm:bool, pub isfullscreen: bool, pub oldstate: bool,
    pub win: xlib::Window
}

impl<'a> PartialEq for Client<'a> {
    fn eq(&self, other: &Client<'a>) -> bool {
        self.win == other.win
    }
}

/**
 * Create a new client from a window ant it's attributes
 */
pub fn createClient(win: xlib::Window, wa: &xlib::XWindowAttributes, monindex: usize) -> Client {
    Client {
        name: "",
        mina: 0.0, maxa: 0.0,
        x: wa.x, y: wa.y, w: wa.width as u32, h: wa.height as u32,
        bw: 0,
        tags: 0,
        isfixed: false, isfloating: false, isurgent: false, neverfocus: false, oldwm: false, isfullscreen: false, oldstate: false,
        win
    }
}

/*
 * Finds the Client containing a Window
 */
// pub fn fromWindow(window : xlib::Window, mons: &'a Vec<Workspace<'a>>) -> Option<&'a Client<'a>> {
//     for m in mons.iter() {
//         for c in m.clients.iter() {
//             if c.win == window {
//                 return Some(c)
//             }
//         }
//     }
//     None
// }

/*
 * Set the window to fullscreen (or not)
 */
// pub fn setfullscreen(client: &Client, dpy: &mut xlib::Display, fullscreen: bool, netatom: &Vec<xlib::Atom>) {
//     println!("dwm-rust : full screen !");
//     if fullscreen && !self.isfullscreen {
//         unsafe { xlib::XChangeProperty(dpy, self.win, netatom[NETWMSTATE], xlib::XA_ATOM, 32, xlib::PropModeReplace, &(netatom[NETWMFULLSCREEN] as u8), 1) };
//         self.isfullscreen = true;
//         self.oldstate = self.isfloating;
//         self.oldbw = self.bw;
//         self.isfloating = true;
//         // self.resize(self.mon.mx, self.mon.my, self.mon.mw, self.mon.mh); TODO
//         unsafe { xlib::XRaiseWindow(dpy, self.win) };
//     } else if !fullscreen && self.isfullscreen {
//         unsafe { xlib::XChangeProperty(dpy, self.win, netatom[NETWMSTATE], xlib::XA_ATOM, 32, xlib::PropModeReplace, &0, 0) };
//         self.isfullscreen = false;
//         self.isfloating = self.oldstate;
//         self.bw = self.oldbw;
//         self.x = self.oldx;
//         self.y = self.oldy;
//         self.w = self.oldw;
//         self.h = self.oldh;
//         // self.resize(self.x, self.y, self.w, self.h); TODO
//         // self.mon.arrange(); TODO
//     }
// }

/**
 * Total width of the window (including borders)
 */
pub fn width(client: &Client) -> u32 {
    client.w + 2 * client.bw
}

/**
 * Total height of the window (including borders)
 */
pub fn height(client: &Client) -> u32 {
    client.h + 2 * client.bw
}

/**
 * Change configuration of the window
 */
pub fn configure<'a>(client: &'a Client<'a>, dpy: &mut xlib::Display) {
    let ce = xlib::XConfigureEvent {
        type_: xlib::ConfigureNotify,
        serial: 0,
        send_event: 1,
        display: dpy,
        event: client.win,
        window: client.win,
        x: client.x,
        y: client.y,
        width: client.w as i32,
        height: client.h as i32,
        border_width: client.bw as i32,
        above: 0,
        override_redirect: 0
    };
    unsafe { xlib::XSendEvent(dpy, client.win, 0, xlib::StructureNotifyMask, &mut xlib::XEvent { configure: ce }) };
}

pub fn draw(c: &Client, dpy: &mut xlib::Display) {
    unsafe { xlib::XMapWindow(dpy, c.win) };
}

/*
 * Gets Atom property
 */
// pub fn getatomprop(&mut self, dpy: &mut xlib::Display, prop: xlib::Atom) -> xlib::Atom {
//     let mut di = 0;
//     let mut dl = 0;
//     let mut p = ptr::null_mut();
//     let mut da = 0; let mut atom = 0;
//     if unsafe { xlib::XGetWindowProperty(dpy, self.win, prop, 0, size_of::<xlib::Atom>() as i64, 0, xlib::XA_ATOM, &mut da, &mut di, &mut dl, &mut dl, &mut p) } == xlib::Success as i32 && !(p.is_null()) {
//         // TODO
//         atom = unsafe { *p } as u64;
//         // xlib::XFree(p); TODO
//     }
//     atom
// }

/*
 * Updates the title
 */
// pub fn updatetitle(&mut self) {
//     // TODO
// }

/*
 * Updates type of the window
 */
// pub fn updatewindowtype(&mut self, dpy: &mut xlib::Display, netatom: &Vec<xlib::Atom>) {
//     let state = self.getatomprop(dpy, netatom[NETWMSTATE]);
//     let wtype = self.getatomprop(dpy, netatom[NETWMWINDOWTYPE]);

//     println!("dwm-rust : atom : {}", state);
//     println!("dwm-rust : netwmfullscreen atom : {}", netatom[NETWMFULLSCREEN]); // Pourquoi pas fullscreen ?

//     if state == netatom[NETWMFULLSCREEN] {
//         self.setfullscreen(dpy, true, netatom);
//     }
//     if wtype == netatom[NETWMWINDOWTYPEDIALOG] {
//         self.isfloating = true;
//     }
// }

/*
 * Updates size hints of the window
 */
// pub fn updatesizehints(&mut self, dpy: &mut xlib::Display) {
//     let mut msize = 0;
//     let mut size = xlib::XSizeHints { // Dummy value
//         flags: 0, x: 0, y: 0, width: 0, height: 0, min_width: 0, min_height: 0, max_width: 0, max_height: 0, width_inc: 0, height_inc: 0, min_aspect: xlib::AspectRatio{x:0, y:0}, max_aspect: xlib::AspectRatio{x:0, y:0}, base_width: 0, base_height: 0, win_gravity: 0
//     };
//     if unsafe { xlib::XGetWMNormalHints(dpy, self.win, &mut size, &mut msize) == 0} {
//         // size is not initialized
//         size.flags = xlib::PSize;
//         if size.flags & xlib::PBaseSize != 0 {
//             self.basew = size.base_width as u32;
//             self.baseh = size.base_height as u32;
//         } else if size.flags & xlib::PMinSize != 0 {
//             self.basew = size.min_width as u32;
//             self.baseh = size.min_height as u32;
//         } else {
//             self.basew = 0;
//             self.baseh = 0;
//         }
//         if size.flags & xlib::PMaxSize != 0 {
//             self.maxw = size.max_width as u32;
//             self.maxh = size.max_height as u32;
//         } else {
//             self.maxw = 0;
//             self.maxh = 0;
//         }
//         if size.flags & xlib::PMinSize != 0 {
//             self.minw = size.min_width as u32;
//             self.minh = size.min_height as u32;
//         } else if size.flags & xlib::PBaseSize != 0 {
//             self.minw = size.base_width as u32;
//             self.maxw = size.base_height as u32;
//         } else {
//             self.minw = 0;
//             self.minh = 0;
//         }
//         if size.flags & xlib::PAspect != 0 {
//             self.mina = size.min_aspect.y as f32 / size.min_aspect.x as f32;
//             self.maxa = size.max_aspect.x as f32 / size.max_aspect.y as f32;
//         } else {
//             self.mina = 0.0;
//             self.maxa = 0.0;
//         }
//         self.isfixed = self.maxw != 0 && self.minw != 0 && self.maxh != 0 && self.minh != 0 && self.maxw == self.minw && self.maxh == self.minh;
//     }
// }

/*
 * Updates the WM Hints
 */
// pub fn updatewmhints(&mut self, dpy: &mut xlib::Display, selmon: &Workspace<'a>) {
//     let wmh = unsafe { xlib::XGetWMHints(dpy, self.win) };
//     if !(wmh.is_null()) {
//         if let Some(sel) = selmon.sel {
//             if self == sel && unsafe { (*wmh).flags } & xlib::XUrgencyHint != 0 {
//                 unsafe { (*wmh).flags &= !xlib::XUrgencyHint };
//                 unsafe { xlib::XSetWMHints(dpy, self.win, wmh) };
//             } else {
//                 self.isurgent = unsafe { (*wmh).flags } & xlib::XUrgencyHint != 0;
//             }
//         }
//         if unsafe { (*wmh).flags } & xlib::InputHint != 0 {
//             self.neverfocus = unsafe { (*wmh).input == 0};
//         } else {
//             self.neverfocus = false;
//         }
//         // xlib::XFree(wmh); TODO
//     }
// }

/*
 * Grabs buttons
 */
// pub fn grabbuttons(&mut self, wm: &mut WM, focused: bool) {
//     wm.updatenumlockmask();
//     let modifiers = vec![0, xlib::LockMask, wm.numlockmask, xlib::LockMask|wm.numlockmask];
//     unsafe { xlib::XUngrabButton(wm.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, self.win) };
//     if focused {
//         for b in config::buttons.iter() {
//             /*if b.click == ClkClientWin {
//             TODO
//         }*/
//         }
//     } else {
//         unsafe { xlib::XGrabButton(wm.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, self.win, 0, (xlib::ButtonPressMask|xlib::ButtonReleaseMask) as u32, xlib::GrabModeAsync, xlib::GrabModeSync, 0, 0) };
//     }
// }

/*
 * Applies the rules
 */
// pub fn applyrules(&mut self) {
//     // TODO
// }

