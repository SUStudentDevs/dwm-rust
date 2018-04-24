extern crate x11;

use std::env;
use std::process;
use std::ptr;

use x11::xlib;

pub mod wm;
pub mod drw;
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

// Fn instead of C macros
fn textw(s: &str, drw: &mut Drw) -> u32 {
    drw.text(0, 0, 0, 0, s, false) as u32 + drw.fonts[0].h
}

fn cleanmask(mask: u32) -> u32 {
    mask
}

union Arg {
    i: i32,
    u: u32,
    f: f32,
    // TODO pointer
}

pub struct Key {
    modif: u32,
    keysym: xlib::KeySym,
    func: fn (&Arg, &mut WM),
    arg: Arg
}

pub struct Button {
    click: u32,
    mask: u32,
    button: u32,
    func: fn (Arg, &mut WM),
    arg: Arg
}

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
        unsafe { xlib::XCloseDisplay(wm.dpy) };
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

/// Checks for another WM running. If there is one, prints an error and exits.
fn checkotherwm(dpy: *mut xlib::Display) { 
    unsafe { 
        xlib::XSetErrorHandler(Some(xerrorstart));
        xlib::XSelectInput(dpy, xlib::XDefaultRootWindow(dpy), xlib::SubstructureRedirectMask);
        xlib::XSync(dpy, 0); xlib::XSetErrorHandler(Some(xerror)); xlib::XSync(dpy, 0); 
    } 
}

/// Setup everything
fn setup(dpy: &mut xlib::Display) -> WM {
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

    let mut wm : WM = WM::new(dpy, drw, screen, root, sw, sh);
    wm.updategeom();
    wm.updatebars();
    wm.updatestatus();
    unsafe {
        xlib::XChangeProperty(wm.dpy, wm.root, wm.netatom[NETSUPPORTED], xlib::XA_ATOM, 32, xlib::PropModeReplace, &(wm.netatom[0] as u8), NETLAST as i32);
        xlib::XDeleteProperty(wm.dpy, wm.root, wm.netatom[NETCLIENTLIST]); 
        xlib::XChangeWindowAttributes(wm.dpy, wm.root, xlib::CWEventMask|xlib::CWCursor, &mut xlib::XSetWindowAttributes {
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

fn wintomon<'a>(wm: &'a mut WM<'a>, w: xlib::Window) -> &'a mut Monitor<'a> {
    // TODO
    if w == wm.root {
        // TODO recttomon
    }
    &mut wm.mons[0]
}

/// Main loop
fn run(wm: &mut WM) {
    let ev = &mut xlib::XEvent { any: xlib::XAnyEvent { type_: 0, serial: 0, send_event: 0, display: wm.dpy, window: wm.root } }; // Dummy value
    unsafe {
        xlib::XSync(wm.dpy, 0);
        while wm.running && xlib::XNextEvent(wm.dpy, ev)==0 {
            handleevent(wm, ev);
        }
    }
}

/// Handle an event
fn handleevent(wm: &mut WM, ev: &xlib::XEvent) {
    unsafe {
        match ev.type_ {
            xlib::ButtonPress => buttonpress(wm, ev),
            xlib::KeyPress => keypress(wm, ev),
            // TODO : les autres handlers
            _ => ()
        }
    }
}

/// Handle a button press
fn buttonpress(wm: &mut WM, e: &xlib::XEvent) {
    let arg = Arg {i: 0};
    let ev = unsafe { e.button };
    // click = CLKROOTWIN;
    // let m = wintomon(ev.window);
    // TODO
}

/// Handle a key press
fn keypress(wm: &mut WM, e: &xlib::XEvent) {
    let ev = unsafe { e.key };
    let keysym = unsafe { xlib::XKeycodeToKeysym(wm.dpy, ev.keycode as u8, 0) };
    for i in 0..config::keys.len() {
        if keysym == config::keys[i].keysym
        && cleanmask(ev.state) == cleanmask(config::keys[i].modif) {
            let func = config::keys[i].func;
            func(&config::keys[i].arg, wm);
        }
    }
}

/// Quit the wm
fn quit(_: &Arg, wm: &mut WM) {
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

/// Cleanup everything
fn cleanup(wm: &mut WM) {
    // TODO
}
