extern crate x11;

use std::ffi::CString;

use x11::xlib;
use x11::keysym;

/// Workspace module
pub mod workspace;
/// Client module
pub mod client;

use { CURNORMAL, SCHEMENORM, isuniquegeom };
use wm::workspace::Workspace;
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
    pub wss: Vec<Workspace<'a>>,
    pub selwsindex: usize,
    pub sw: u32, pub sh: u32,
    pub bh: u32,
    pub stext: String,
    pub numlockmask: u32,
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
        wss: Vec::new(),
        selwsindex: 0,
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

/**
 * Create all the workspaces and set their data
 */
pub fn createWorkspaces(wm: WM) -> WM {
    let mut wm = wm;
    if wm.wss.is_empty() {
        return WM {
            wss:config::tags.iter().map(|t| {
                let ws = workspace::createWorkspace(t);
                workspace::updateBarPos(workspace::Workspace {
                    w: wm.sw, h: wm.sh,
                    ..ws
                }, wm.bh)
            }).collect(),
            selwsindex: 4,
            ..wm
        }
    }
    wm
}

/**
 * Updates the status bars
 */
pub fn updateBars(wm: WM) -> WM {
    let mut wm = wm;
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
        cursor: 0
    };
    if wm.wss[0].barwin == 0 {
        let barwin = unsafe {
            xlib::XCreateWindow(wm.drw.dpy,
                                wm.root,
                                wm.wss[0].x, wm.wss[0].by, wm.wss[0].w as u32,
                                wm.bh,
                                0,
                                xlib::XDefaultDepth(wm.drw.dpy, wm.screen),
                                xlib::CopyFromParent as u32,
                                xlib::XDefaultVisual(wm.drw.dpy, wm.screen),
                                xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask,
                                &mut wa) };
        unsafe { xlib::XDefineCursor(wm.drw.dpy, barwin, wm.cursor[CURNORMAL].cursor) };
        unsafe { xlib::XMapRaised(wm.drw.dpy, barwin) };
        for ws in wm.wss.iter_mut() {
            if ws.barwin == 0 { ws.barwin = barwin };
        }
    }
    wm
}

/**
 * Updates the status bar text
 */
pub fn updateStatus(wm: WM) -> WM{
    // if(...) TODO
    WM {drw: workspace::drawBar(wm.drw, wm.bh, &wm.scheme, &wm.wss, wm.selwsindex, &wm.stext[..]), ..wm}
}

/**
 * Loads and grabs the keys defined in config::keys
 */
pub fn grabKeys(wm: WM) -> WM {
    let wm = updatenumlockmask(wm);
    let modifiers = vec![0, xlib::LockMask, wm.numlockmask, wm.numlockmask|xlib::LockMask];

    unsafe { xlib::XUngrabKey(wm.drw.dpy, xlib::AnyKey, xlib::AnyModifier, wm.root) };
    for i in 0..config::keys.len() {
        let code = unsafe { xlib::XKeysymToKeycode(wm.drw.dpy, config::keys[i].keysym) };
        if code != 0 {
            for j in 0..modifiers.len() {
                unsafe { xlib::XGrabKey(wm.drw.dpy, code as i32, config::keys[i].modif | modifiers[j], wm.root, 1, xlib::GrabModeAsync, xlib::GrabModeAsync) };
            }
        }
    }
    wm
}

// /**
//  * Grabs buttons
//  */
// pub fn grabbuttons(&mut wm, c: &Client, focused: bool) {
//     wm.updatenumlockmask();
//     let modifiers = vec![0, xlib::LockMask, wm.numlockmask, xlib::LockMask|wm.numlockmask];
//     unsafe { xlib::XUngrabButton(wm.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, c.win) };
//     if focused {
//         for b in config::buttons.iter() {
//             /*if b.click == ClkClientWin {
//             TODO
//         }*/
//         }
//     } else {
//         unsafe { xlib::XGrabButton(wm.drw.dpy, xlib::AnyButton as u32, xlib::AnyModifier, c.win, 0, (xlib::ButtonPressMask|xlib::ButtonReleaseMask) as u32, xlib::GrabModeAsync, xlib::GrabModeSync, 0, 0) };
//     }
// }

fn updatenumlockmask(wm: WM) -> WM {
    let modmap = unsafe { (*xlib::XGetModifierMapping(wm.drw.dpy)) };
    let modifiermap = unsafe { Vec::from_raw_parts(modmap.modifiermap, 8 * modmap.max_keypermod as usize, 8 * modmap.max_keypermod as usize) };
    for i in 0..8 {
        for j in 0..modmap.max_keypermod {
            if modifiermap[(i * modmap.max_keypermod + j) as usize] == unsafe { xlib::XKeysymToKeycode(wm.drw.dpy, keysym::XK_Num_Lock as u64) } {
                return WM {
                    numlockmask: 1 << i,
                    ..wm
                }
            }
        }
    }
    WM { numlockmask:0, ..wm}
    // unsafe { xlib::XFreeModifiermap(&mut modmap); } TODO Causes a crash for some reason
}

/**
 * Manage a new Window
 */
pub fn manage<'a>(wm: WM<'a>, w: xlib::Window, wa: &xlib::XWindowAttributes) -> WM<'a> {
    let c = client::createClient(w, wa, wm.selwsindex);
    // c.updatetitle();
    // let mut trans = 0;
    // if unsafe { xlib::XGetTransientForHint(wm.drw.dpy, w, &mut trans) } != 0 {
    //     if let Some(t) = Client::from(trans, &wm.mons) {
    //         c.monindex = t.monindex;
    //         c.tags = t.tags;
    //     } else {
    //         c.monindex = wm.selmonindex;
    //         c.applyrules();
    //     }
    // } else {
    //     c.monindex = wm.selmonindex;
    //     c.applyrules();
    // }
    // let mon = &wm.mons[c.monindex];
    // if c.x + c.width() as i32 > mon.mx + mon.mw as i32 {
    //     c.x = mon.mx + mon.mw as i32 - c.width() as i32;
    // }
    // if c.y + c.height() as i32 > mon.my + mon.mw as i32 {
    //     c.y = mon.my + mon.mw as i32 - c.height() as i32;
    // }
    // c.x = c.x.max(mon.mx);
    // c.y = c.y.max(if mon.by == mon.my && c.x + (c.w/2) as i32 >= mon.wx && c.x + ((c.w/2) as i32) < mon.wx + mon.ww as i32 { wm.bh as i32 } else { mon.my });
    // let mut wc = xlib::XWindowChanges {
    //     x: 0, y: 0, width:0, height: 0, border_width: c.bw as i32, sibling: 0, stack_mode: 0
    // };
    // unsafe { xlib::XConfigureWindow(wm.drw.dpy, w, xlib::CWBorderWidth as u32, &mut wc) };
    // unsafe { xlib::XSetWindowBorder(wm.drw.dpy, w, wm.scheme[SCHEMENORM].border.pix) };
    // c.configure(wm.drw.dpy);
    // c.updatewindowtype(wm.drw.dpy, &wm.netatom);
    // c.updatesizehints(wm.drw.dpy);
    // c.updatewmhints(wm.drw.dpy, &wm.mons[wm.selmonindex]);
    // unsafe { xlib::XSelectInput(wm.drw.dpy, w, xlib::EnterWindowMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::StructureNotifyMask) };
    // // wm.grabbuttons(&c, false); TODO
    // if !c.isfloating {
    //     c.isfloating = trans != 0 || c.isfixed;
    //     c.oldstate = c.isfloating;
    // }
    // if c.isfloating {
    //     unsafe { xlib::XRaiseWindow(wm.drw.dpy, c.win) };
    // }
    // TODO
    client::draw(&c, wm.drw.dpy);
    wm
    // focus(None) TODO
}
