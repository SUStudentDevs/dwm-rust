#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate x11;

use std::env;
use std::process;
use std::ptr;
use std::process::Command;

use x11::{ xlib, xinerama };

/// Events handling
pub mod events;
/// Window Manager module
pub mod wm;
/// Drawable module
pub mod drw;
/// Configuration module
pub mod config;

use events::handleEvent;

use wm::WM;
use wm::workspace;
use wm::workspace::Layout;
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
    func: for<'b> fn (&Arg, WM<'b>) -> WM<'b>,
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
    func: for<'b> fn (&Arg, WM<'b>) -> WM<'b>,
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
        checkOtherWm(dpy);
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
pub fn checkOtherWm(dpy: *mut xlib::Display) {
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

    let wm = wm::updateStatus(wm::updateBars(wm::createWorkspaces(wm::initWm(drw, screen, root, sw, sh))));
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
    executeStartCmds(wm::setRootBackground(wm::grabKeys(wm)))
}

pub fn isUniqueGeom(unique: &Vec<xinerama::XineramaScreenInfo>, n: usize, info: &xinerama::XineramaScreenInfo) -> bool {
    for i in n..0 {
        if unique[i].x_org == info.x_org && unique[i].y_org == info.y_org && unique[i].width == info.width && unique[i].height == info.height {
            return false
        }
    }
    true
}

pub fn executeStartCmds(wm: WM) -> WM {
    config::startCmds.into_iter().map(|s| {Arg {s}}).fold(wm, |wm, a| { spawn(&a, wm) })
}

/**
 * Main program loop
 */
pub fn run(mut wm: WM) -> WM {
    let ev = &mut xlib::XEvent {
        any: xlib::XAnyEvent {
            type_: 0,
            serial: 0,
            send_event: 0,
            display: wm.drw.dpy,
            window: wm.root
        }
    }; // Dummy value

    unsafe {
        xlib::XSync(wm.drw.dpy, 0);
        while wm.running && xlib::XNextEvent(wm.drw.dpy, ev) == 0 {
            wm = handleEvent(wm, ev);
        }
    }
    wm
}
/**
 * Executes a shell command
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing the command (&str) to execute
 * * `wm` - Window Manager
 */
pub fn spawn<'a>(arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let v : Vec<&str> = unsafe { arg.s.split(' ').collect() };
    let mut command = Command::new(v[0]);
    for i in 1..v.len() {
        command.arg(v[i]);
    }
    command.spawn().expect(&["Command", unsafe { arg.s }, "has failed..."].join(" ")[..]);
    wm
}

/**
 * Change to another Workspace
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing the number (u32) of the Workspace to switch to
 * * `wm` - Window Manager
 */
pub fn changeWs<'a>(arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let index = unsafe { arg.u };
    if index > 0 && index <= wm.wss.len() as u32 && (index-1) != wm.selwsindex as u32 {
        workspace::hideAllClients(&wm.wss[wm.selwsindex], wm.drw.dpy);
        let wm = wm::updateStatus(WM {
            selwsindex: (index-1) as usize,
            oldwsindex: wm.selwsindex,
            ..wm
        });
        workspace::showAllClients(&wm.wss[wm.selwsindex], wm.drw.dpy);
        wm
    } else {
        wm
    }
}

/**
 * Change to another Workspace relative to the current one
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing the number (i32) of positions
 * *    to go forward (>0) or backwards (<0)
 * * `wm` - Window Manager
 */
pub fn changeWsRel<'a>(arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let index = unsafe { arg.i } + wm.selwsindex as i32;

    if index < 0 {
        let wm = changeWs(&Arg {u: wm.wss.len() as u32}, wm);
        wm
    } else if index >= wm.wss.len() as i32 {
        let wm = changeWs(&Arg {u: 1}, wm);
        wm
    } else {
        let wm = changeWs(&Arg {u: index as u32 + 1}, wm);
        wm
    }
}

/**
 * Change to the previously selected Workspace
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing whatever
 * * `wm` - Window Manager
 */
pub fn pivotWs<'a>(_arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let wm = changeWs(&Arg {u: wm.oldwsindex as u32 + 1}, wm);
    wm
}

/**
 * Moves a Client to another Workspace
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing the number (u32) of the Workspace to move the client to
 * * `wm` - Window Manager
 */
pub fn moveClientToWs<'a>(arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let index = (unsafe { arg.u } - 1) as usize;
    if index < wm.wss.len() as usize && index != wm.selwsindex as usize {
        let (mut wm, w) = wm::findPointedWindow(wm);
        {
            for i in 0..wm.wss[wm.selwsindex].clients.len() {
                if wm.wss[wm.selwsindex].clients[i].win == w {
                    let c = wm.wss[wm.selwsindex].clients.remove(i);
                    wm.wss[index].clients.insert(0, c);
                    break;
                }
            }
        }
        let ws = workspace::updateGeom(wm.wss.remove(wm.selwsindex), wm.drw.dpy);
        wm.wss.insert(wm.selwsindex, ws);
        let ws = workspace::updateGeom(wm.wss.remove(index), wm.drw.dpy);
        workspace::hideAllClients(&ws, wm.drw.dpy);
        wm.wss.insert(index, ws);
        wm::updateStatus(wm)
    } else {
        wm
    }
}

/**
 * Move a Client to another Workspace relative to the current one
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing the number (i32) of positions
 * *    to move the client forward (>0) or backwards (<0)
 * * `wm` - Window Manager
 */
pub fn moveClientToWsRel<'a>(arg: &Arg, wm: WM<'a>) -> WM<'a> {
    let index = unsafe { arg.i } + wm.selwsindex as i32;

    if index < 0 {
        let wm = moveClientToWs(&Arg {u: wm.wss.len() as u32}, wm);
        wm
    } else if index >= wm.wss.len() as i32 {
        let wm = moveClientToWs(&Arg {u: 1}, wm);
        wm
    } else {
        let wm = moveClientToWs(&Arg {u: index as u32 + 1}, wm);
        wm
    }
}

/**
 * Move a client to the previously selected Workspace
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing whatever
 * * `wm` - Window Manager
 */
pub fn pivotClientToWs<'a>(_: &Arg, wm: WM<'a>) -> WM<'a> {
    let wm = moveClientToWs(&Arg {u: wm.oldwsindex as u32 + 1}, wm);
    wm
}

/**
 * Closes a Client
 *
 * # Arguments
 * * `arg` - Reference to an Arg containing whatever
 * * `wm` - Window Manager
 */
pub fn closeClient<'a>(_: &Arg, wm: WM<'a>) -> WM<'a> {
    let (mut wm, w) = wm::findPointedWindow(wm);
    for i in 0..wm.wss.len() {
        let ws = &mut wm.wss[i];
        for i in 0..ws.clients.len() {
            if ws.clients[i].win == w {
                let c = ws.clients.remove(i);
                client::destroyClient(c, wm.drw.dpy);
                break;
            }
        }
    }
    wm
}


/**
 * Quits the WM
 */
pub fn quit<'a>(_: &Arg, wm: WM<'a>) -> WM<'a> {
    WM {
        running: false,
        ..wm
    }
}

/**
 * Cleanup and frees memory
 */
fn cleanup(wm: WM) -> WM {
    // TODO
    wm
}
