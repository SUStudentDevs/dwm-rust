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
use wm::monitor::{ Monitor, Layout };
use wm::client::Client;
use drw::Drw;

const VERSION: &str = "0.0.1";

// WM Atom indexes
const WMPROTOCOLS: usize = 0; const WMDELETE: usize = 1; const WMSTATE: usize  = 2; const WMTAKEFOCUS: usize = 3; const WMLAST: usize = 4;
// Net Atom indexes
const NETACTIVEWINDOW: usize = 0; const NETSUPPORTED: usize = 1; const NETWMNAME: usize = 2; const NETWMSTATE: usize = 3; const NETWMFULLSCREEN: usize = 4; const NEWWMWINDOWTYPE: usize = 5; const NETWMWINDOWTYPEDIALOG: usize = 6; const NETCLIENTLIST: usize = 7; const NETLAST: usize = 8;
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
 * Stores a button for click events
 */
pub struct Button<'a> {
    click: u32,
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
    ltidxs: Vec<Vec<&'a Layout<'a>>>, // Matrix of tags and yaouts
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
        let mut wm = setup(dpy);
        run(&mut wm); 
        cleanup(&mut wm);
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
    let mut drw = Drw::new(dpy, screen, root, sw, sh);

    drw.load_fonts(config::fonts.to_vec());
    if drw.fonts.len()<1 {
        eprintln!("no fonts could be loaded.\n");
        process::exit(1);
    }

    let mut wm : WM = WM::new(drw, screen, root, sw, sh);
    wm.updategeom();
    wm.updatebars();
    wm.updatestatus();
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
    wm.grabkeys();
    // focus(None); TODO
    wm
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
pub fn run(wm: &mut WM<'static>) {
    let ev = &mut xlib::XEvent { any: xlib::XAnyEvent { type_: 0, serial: 0, send_event: 0, display: wm.drw.dpy, window: wm.root } }; // Dummy value
    unsafe {
        xlib::XSync(wm.drw.dpy, 0);
        while wm.running && xlib::XNextEvent(wm.drw.dpy, ev)==0 {
            handleevent(wm, ev);
        }
    }
}

/**
 * Handles an event
 */
pub fn handleevent(wm: &mut WM<'static>, ev: &xlib::XEvent) {
    unsafe {
        match ev.type_ {
            xlib::ButtonPress => buttonpress(wm, ev),
            xlib::ConfigureRequest => configurerequest(wm, ev),
            xlib::EnterNotify => enternotify(wm, ev),
            xlib::KeyPress => keypress(wm, ev),
            // TODO : les autres handlers
            _ => ()
        }
    }
}

/**
 * Handles a button press
 */
pub fn buttonpress(wm: &mut WM, e: &xlib::XEvent) {
    let arg = Arg {i: 0};
    let ev = unsafe { e.button };
    // click = CLKROOTWIN;
    // let m = wintomon(ev.window);
    // TODO
}

/**
 * Handles a change of a window configuration
 */
pub fn configurerequest(wm: &mut WM<'static>, e: &xlib::XEvent) {
    let ev = unsafe { e.configure_request };

    if let Some(c) = Client::from(ev.window, &mut (wm.mons)) {
        if ev.value_mask & xlib::CWBorderWidth as u64 != 0 {
            c.bw = ev.border_width;
        } else if c.isfloating /*|| !(wm.mons[wm.selmonindex].lt[wm.mons[wm.selmonindex].sellt as usize].arrange) TODO*/ {
            let m = &(c.mon);
            // TODO
        }
    } else {
        let mut wc = xlib::XWindowChanges {
            x: ev.x, y: ev.y,
            width: ev.width, height: ev.height,
            border_width: ev.border_width,
            sibling: ev.above,
            stack_mode: ev.detail
        };
        unsafe { xlib::XConfigureWindow(wm.drw.dpy, ev.window, ev.value_mask as u32, &mut wc) };
    }
    unsafe { xlib::XSync(wm.drw.dpy, 0) };
}

/**
 * Handles the entering of a window
 */
pub fn enternotify(wm: &mut WM<'static>, e: &xlib::XEvent) {
    let ev = unsafe { e.crossing };
    if (ev.mode != xlib::NotifyNormal || ev.detail == xlib::NotifyInferior) && ev.window != wm.root {
        return;
    }
    /*if let Some(c) = Client::from(ev.window, &mut wm.mons) { TODO
        let m = &c.mon;
    } else {
        let m = Monitor::from_window(ev.window, wm.root, &mut wm.mons, &mut wm.mons[wm.selmonindex]);
    }*/
}

/**
 * Handles a keypress
 */
pub fn keypress(wm: &mut WM, e: &xlib::XEvent) {
    let ev = unsafe { e.key };
    let keysym = unsafe { xlib::XKeycodeToKeysym(wm.drw.dpy, ev.keycode as u8, 0) };
    for i in 0..config::keys.len() {
        if keysym == config::keys[i].keysym
        && cleanmask(ev.state) == cleanmask(config::keys[i].modif) {
            let func = config::keys[i].func;
            func(&config::keys[i].arg, wm);
        }
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
fn tilearrange(monitor: &Monitor) {
    // TODO
}

fn monoclearrange(monitor: &Monitor) {
    // TODO
}

fn noarrange(monitor: &Monitor) {
    // Nothing
}

fn gridarrange(monitor: &Monitor) {
    // TODO
}

/**
 * Cleanup and free memory
 */
fn cleanup(wm: &mut WM) {
    // TODO
}
