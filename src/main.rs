#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate x11;

use std::env;
use std::process;
use std::ptr;
use std::process::Command;

use x11::{ xlib, xinerama };

/// Window Manager module
pub mod wm;
/// Drawable module
pub mod drw;
/// Configuration module
pub mod config;

use wm::WM;
use wm::workspace::{ Layout, Workspace };
use wm::client;
use wm::client::Client;

const VERSION: &str = "0.0.1";

// WM Atom indexes
const WMPROTOCOLS: usize = 0; const WMDELETE: usize = 1; const WMSTATE: usize  = 2; const WMTAKEFOCUS: usize = 3; const WMLAST: usize = 4;
// Net Atom indexes
const NETACTIVEWINDOW: usize = 0; const NETSUPPORTED: usize = 1; const NETWMNAME: usize = 2; const NETWMSTATE: usize = 3; const NETWMFULLSCREEN: usize = 4; const NETWMWINDOWTYPE: usize = 5; const NETWMWINDOWTYPEDIALOG: usize = 6; const NETCLIENTLIST: usize = 7; const NETLAST: usize = 8;
// Cursor indexes
pub const CURNORMAL: usize = 0; pub const CURRESIZE: usize = 1; pub const CURMOVE: usize = 2;
// Color scheme indexes
pub const SCHEMENORM: usize = 0; pub const SCHEMESEL: usize = 1;

fn cleanmask(mask: u32) -> u32 {
    mask
}

/**
 * Stores an argument to pass to functions on keypress and click events
 */
pub union Arg<'a> {
    i: i32,
    u: u32,
    f: f32,
    s: &'a str
}

/**
 * Stores a key for keypress events
 */
pub struct Key<'a> {
    modif: u32,
    keysym: xlib::KeySym,
    func: fn (&Arg, &mut WM),
    arg: Arg<'a>
}

/**
 * Different types of click events
 */
pub enum Click {
    ClkTagBar, ClkLtSymbol, ClkStatusText, ClkWinTitle, ClkClientWin, ClkRootWin, ClkLast
}

/**
 * Stores a button for click events
 */
pub struct Button<'a> {
    click: Click,
    mask: u32,
    button: u32,
    func: fn (Arg, &mut WM),
    arg: Arg<'a>
}

/**
 * Stores a tag
 */
pub struct Pertag<'a> {
    curtag: u32, prevtag: u32,  // Current and previous tag
    nmasters: Vec<i32>, // number windows in master area
    mfacts: Vec<f32>,   // mfacts per tag
    selltds: Vec<u32>,  // Selected layouts
    ltidxs: Vec<Vec<&'a Layout<'a>>>, // Matrix of tags and layouts
    showbars: Vec<bool>,    // Display bar for each tag
    prefzooms: Vec<&'a Client<'a>> // Zoom information
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len()==2 && args[1]==String::from("-v") {
        println!("dwm-rust-{}", ::VERSION);
        process::exit(0);
    } if args.len()>1 {
        println!("usage: dwm-rust [-v]");
        process::exit(1);
    }
    if unsafe { xlib::XSupportsLocale() } == 0 {
        println!("Warning : no locale support");
    } if let Some(dpy) = Some( unsafe { &mut(*xlib::XOpenDisplay(ptr::null())) }) {
        // This is where we'll work
        checkotherwm(dpy);
        let wm = cleanup(run(setup(dpy)));
        unsafe { xlib::XCloseDisplay(wm.drw.dpy) };
    } else {
        println!("dwm-rust: can't open display");
        process::exit(1);
    }
}

/// Prints an X error on start of the wm and exits the program.
extern "C" fn xerrorstart(_dpy: *mut xlib::Display, _ee: *mut xlib::XErrorEvent) -> i32 {
    println!("dwm-rust: another window manager is already running\n");
    process::exit(1);
}

/// Handles errors. TODO : completer les cas sans erreur
unsafe extern "C" fn xerror(_dpy: *mut xlib::Display, ee: *mut xlib::XErrorEvent) -> i32 {
    if (*ee).error_code == xlib::BadWindow
    || (*ee).error_code == xlib::BadDrawable
    || (*ee).error_code == xlib::BadMatch
    || (*ee).error_code == xlib::BadAccess {
        0
    } else {
        eprintln!("dwm-rust: fatal error: request code={}, error code={}", (*ee).request_code, (*ee).error_code);
        process::exit(1);
    }
}

/**
 * Checks for another WM running. If there is one, prints an error and exits.
 */
pub fn checkotherwm(dpy: *mut xlib::Display) {
    unsafe {
        xlib::XSetErrorHandler(Some(xerrorstart));
        xlib::XSelectInput(dpy, xlib::XDefaultRootWindow(dpy), xlib::SubstructureRedirectMask);
        xlib::XSync(dpy, 0); xlib::XSetErrorHandler(Some(xerror)); xlib::XSync(dpy, 0);
    }
}

/**
 * Setup the Window Manager
 */
pub fn setup(dpy: &mut xlib::Display) -> WM {
    let screen = unsafe { xlib::XDefaultScreen(dpy) };
    let sw = unsafe { xlib::XDisplayWidth(dpy, screen) } as u32;
    let sh = unsafe { xlib::XDisplayHeight(dpy, screen) } as u32;
    let root = unsafe { xlib::XRootWindow(dpy, screen) };
    let drw = drw::createDrw(dpy, screen, root, sw, sh);

    let drw = drw::loadFonts(drw, config::fonts.to_vec());
    if drw.fonts.len()<1 {
        eprintln!("no fonts could be loaded.\n");
        process::exit(1);
    }

    let mut wm = wm::updateStatus(wm::updateBars(wm::createWorkspaces(wm::initWm(drw, screen, root, sw, sh))));
    unsafe {
        xlib::XChangeProperty(wm.drw.dpy, wm.root, wm.netatom[NETSUPPORTED], xlib::XA_ATOM, 32, xlib::PropModeReplace, &(wm.netatom[0] as u8), NETLAST as i32);
        xlib::XDeleteProperty(wm.drw.dpy, wm.root, wm.netatom[NETCLIENTLIST]);
        xlib::XChangeWindowAttributes(wm.drw.dpy, wm.root, xlib::CWEventMask|xlib::CWCursor, &mut xlib::XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0,
            border_pixmap: xlib::CopyFromParent as u64,
            border_pixel: 0,
            bit_gravity: xlib::ForgetGravity,
            win_gravity: xlib::NorthWestGravity,
            backing_store: xlib::NotUseful,
            backing_planes: 1,
            backing_pixel: 0,
            save_under: 0,
            event_mask: xlib::SubstructureRedirectMask|xlib::SubstructureNotifyMask|xlib::ButtonPressMask|xlib::PointerMotionMask|xlib::EnterWindowMask|xlib::LeaveWindowMask|xlib::StructureNotifyMask|xlib::PropertyChangeMask,
        do_not_propagate_mask: 0,
            override_redirect: 0,
            colormap: xlib::CopyFromParent as u64,
            cursor: wm.cursor[CURNORMAL].cursor
        });
    }
    // focus(None); TODO
    wm::grabKeys(wm)
}

pub fn isuniquegeom(unique: &Vec<xinerama::XineramaScreenInfo>, n: usize, info: &xinerama::XineramaScreenInfo) -> bool {
    for i in n..0 {
        if unique[i].x_org == info.x_org && unique[i].y_org == info.y_org && unique[i].width == info.width && unique[i].height == info.height {
            return false
        }
    }
    true
}

/**
 * Main program loop
 */
pub fn run(wm: WM) -> WM {
    let ev = &mut xlib::XEvent { any: xlib::XAnyEvent { type_: 0, serial: 0, send_event: 0, display: wm.drw.dpy, window: wm.root } }; // Dummy value
    let mut wm = wm;
    unsafe {
        xlib::XSync(wm.drw.dpy, 0);
        println!("Salut les amis");
        while wm.running && xlib::XNextEvent(wm.drw.dpy, ev) == 0 {
            wm = handleEvent(wm, ev);
        }
    }
    wm
}

/**
 * Handles an event
 */
pub fn handleEvent<'a>(wm: WM<'a>, ev: &xlib::XEvent) -> WM<'a> {
    unsafe {
        match ev.type_ {
            //xlib::ButtonPress => buttonpress(wm, ev),
            //xlib::ConfigureRequest => configurerequest(wm, ev),
            //xlib::EnterNotify => enternotify(wm, ev),
            xlib::KeyPress => keyPress(wm, ev),
            xlib::MapRequest => mapRequest(wm, ev),
            // TODO : les autres handlers
            _ => wm
        }
    }
}

// /**
//  * Handles a ButtonPress event
//  */
// pub fn buttonpress(wm: &mut WM, e: &xlib::XEvent) {
//     let arg = Arg {i: 0};
//     let ev = unsafe { e.button };
//     // click = CLKROOTWIN;
//     // let m = wintomon(ev.window);
//     // TODO
// }

// /**
//  * Handles a ConfigureRequest event
//  */
// pub fn configurerequest(wm: &mut WM, e: &xlib::XEvent) {
//     let ev = unsafe { e.configure_request };
//     if Client::from(ev.window, &wm.mons) == None {
//         let mut wc = xlib::XWindowChanges {
//             x: ev.x, y: ev.y,
//             width: ev.width, height: ev.height,
//             border_width: ev.border_width,
//             sibling: ev.above,
//             stack_mode: ev.detail
//         };
//         unsafe { xlib::XConfigureWindow(wm.drw.dpy, ev.window, ev.value_mask as u32, &mut wc) };
//     } else {
//         for m in wm.mons.iter_mut() {
//             for c in m.clients.iter_mut() {
//                 if c.win == ev.window {
//                     if ev.value_mask & xlib::CWBorderWidth as u64 != 0 {
//                         c.bw = ev.border_width as u32;
//                     } else if c.isfloating /*|| !(wm.selmon.lt[wm.selmon.sellt as usize].arrange) TODO*/ {
//                     // let m = &mut wm.mons[c.monindex];
//                     // TODO
//                     } else {
//                         c.configure(wm.drw.dpy);
//                     }
//                 }
//             }
//         }
//     }
//     unsafe { xlib::XSync(wm.drw.dpy, 0) };
// }

// /**
//  * Handles an EnterNotify event
//  */
// pub fn enternotify(wm: &mut WM, e: &xlib::XEvent) {
//     let ev = unsafe { e.crossing };
//     if (ev.mode != xlib::NotifyNormal || ev.detail == xlib::NotifyInferior) && ev.window != wm.root {
//         return;
//     }
//     let mut c = Client::from(ev.window, &wm.mons);
//     let m = if let Some(ref mut cl) = c {
//         &wm.mons[cl.monindex]
//     } else {
//         Workspace::from_window(ev.window, wm.root, &wm.mons, &wm.mons[wm.selmonindex])
//     };
//     if m != &wm.mons[wm.selmonindex] {
//         // unfocus(selmon.sel, true); TODO
//         wm.selmonindex = m.num as usize;
//     } else {
//         let mut c = Client::from(ev.window, &wm.mons);
//         let selmon = &wm.mons[wm.selmonindex];
//         match (c, selmon.sel) {
//             (None, _) => return,
//             (Some(cl), Some(sel)) => if cl == sel { return },
//             _ => ()
//         }
//     }
//     if let Some(cl) = c {
//         // focus(cl); TODO
//     }
// }

/**
 * Handles a KeyPress event
 */
pub fn keyPress<'a>(mut wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.key };
    let keysym = unsafe { xlib::XKeycodeToKeysym(wm.drw.dpy, ev.keycode as u8, 0) };
    for i in 0..config::keys.len() {
        if keysym == config::keys[i].keysym
        && cleanmask(ev.state) == cleanmask(config::keys[i].modif) {
            let func = config::keys[i].func;
            func(&config::keys[i].arg, &mut wm);
        }
    }
    wm
}

/**
 * Handles a MapRequest event
 */
pub fn mapRequest<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.map_request };
    let mut wa = xlib::XWindowAttributes { // Dummy value
        x: 0, y: 0, width: 0, height: 0, border_width: 0, depth: 0, visual: ptr::null_mut(), root: wm.root, class: 0, bit_gravity: 0, win_gravity: 0, backing_store: 0, backing_planes: 0, backing_pixel: 0, save_under: 0, colormap: 0, map_installed: 0, map_state: 0, all_event_masks: 0, your_event_mask: 0, do_not_propagate_mask: 0, override_redirect: 0, screen: ptr::null_mut()
    };
    if unsafe { xlib::XGetWindowAttributes(wm.drw.dpy, ev.window, &mut wa) } == 0 || wa.override_redirect != 0 {
        wm
    } else if client::findFromWindow(ev.window, &wm.wss) == None {
        return wm::manage(wm, ev.window, &wa);
    } else {
        wm
    }
}

/**
 * Execute a shell command
 */
pub fn spawn(arg: &Arg, _: &mut WM) {
    let v : Vec<&str> = unsafe { arg.s.split(' ').collect() };
    let mut command = Command::new(v[0]);
    for i in 1..v.len() {
        command.arg(v[i]);
    }
    command.spawn().expect(&["Command", unsafe { arg.s }, "has failed..."].join(" ")[..]);
}

/**
 * Quit the WM
 */
pub fn quit(_: &Arg, wm: &mut WM) {
    wm.running = false;
}

// Arrange functions
fn tilearrange(workspace: &Workspace) {
    // TODO
}

fn monoclearrange(workspace: &Workspace) {
    // TODO
}

fn noarrange(workspace: &Workspace) {
    // Nothing
}

fn gridarrange(workspace: &Workspace) {
    // TODO
}

/**
 * Cleanup and free memory
 */
fn cleanup(wm: WM) -> WM {
    // TODO
    wm
}
