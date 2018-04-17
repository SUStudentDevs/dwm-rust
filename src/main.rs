extern crate x11;

pub mod drw;

use std::env;
use std::process;
use std::ptr;
use x11::xlib;

const VERSION: &str = "0.0.1";

static mut running: bool = true;

enum Arg {
    Int(i32),
    UnsignedInt(u32),
    Float(f32),
    // TODO pointer
}

struct Button {
    click: u32,
    mask: u32,
    button: u32,
    func: fn (*const Arg),
    arg: Arg
}

struct Client<'a> {
    name: String,
    mina: f32, maxa: f32,
    x: i32, y: i32, w: i32, h: i32,
    oldx: i32, oldy: i32, oldw: i32, oldh: i32,
    basew: i32, baseh: i32, inc3: i32, inch: i32, maxw: i32, maxh: i32, minw: i32, minh: i32,
    bw: i32, oldbw: i32,
    tags: u32,
    isfixed: bool, isfloating: bool, isurgent: bool, neverfocus: bool, oldstate:bool, isfullscreen: bool,
    next: Option<&'a Client<'a>>,
    snext: Option<&'a Client<'a>>,
    mon: &'a Monitor<'a>,
    win: xlib::Window
}

struct Layout {
    symbol: String,
    arrange: fn(&Monitor)
}

struct Pertag<'a> {
    curtag: u32, prevtag: u32,  // Current and previous tag
    nmasters: Vec<i32>, // number windows in master area
    mfacts: Vec<f32>,   // mfacts per tag
    selltds: Vec<u32>,  // Selected layouts
    ltidxs: Vec<Vec<&'a Layout>>, // Matrix of tags and yaouts
    showbars: Vec<bool>,    // Display bar for each tag
    prefzooms: Vec<&'a Client<'a>> // Zoom information
}

struct Monitor<'a> {
    ltsymbol: String,
    mfact: f32,
    nmaster: i32,
    num: i32,
    by: i32,    // Bar
    mx: i32, my: i32, mw: i32, mh: i32, // Monitor
    wx: i32, wy: i32, ww: i32, wh: i32, // Window
    seltags: u32,
    sellt: u32,
    tagset: Vec<u32>,
    showbar: bool,
    topbar: bool,
    clients: Option<&'a Client<'a>>,
    sel: Option<&'a Client<'a>>,
    stack: Option<&'a Client<'a>>,
    next: Option<&'a Monitor<'a>>,
    barwin: xlib::Window,
    lt: Vec<Layout>,
    pertag: &'a Pertag<'a>
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len()==2 && args[1]==String::from("-v") {
        println!("dwm-rust-{}", ::VERSION);
        process::exit(0);
    } if args.len()>1 {
        println!("usage: dwm-rust [-v]");
        process::exit(1);
    } if unsafe { xlib::XSupportsLocale() } == 0 {
        println!("Warning : no locale support");
    } if let Some(dpy) = Some( unsafe { xlib::XOpenDisplay(ptr::null()) }) {
        // This is where we'll work
        unsafe { checkotherwm(dpy) };
        unsafe { setup(dpy) };
        unsafe { run(dpy) };
    } else {
        println!("dwm-rust: can't open display");
        process::exit(1);
    }
}

/// Prints an X error on start of the wm and exits the program.
extern "C" fn xerrorstart(_dpy: *mut xlib::Display, _ee: *mut xlib::XErrorEvent) -> i32 {
    println!("dwm: another window manager is already running\n");
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
        eprintln!("dwm: fatal error: request code={}, error code={}",
                  (*ee).request_code, (*ee).error_code);
        process::exit(1);
    }
}

/// Checks for another WM running. If there is one, prints an error and exits.
unsafe fn checkotherwm(dpy: *mut xlib::Display) {
    xlib::XSetErrorHandler(Some(xerrorstart));
    xlib::XSelectInput(dpy, xlib::XDefaultRootWindow(dpy), xlib::SubstructureRedirectMask);
    xlib::XSync(dpy, 0);
    xlib::XSetErrorHandler(Some(xerror));
    xlib::XSync(dpy, 0);
}

/// Setup everything
unsafe fn setup<'a>(dpy: *mut xlib::Display) -> drw::Drw {
    let screen = xlib::XDefaultScreen(dpy);
    let sw = xlib::XDisplayWidth(dpy, screen) as u32;
    let sh = xlib::XDisplayHeight(dpy, screen) as u32;
    let root = xlib::XRootWindow(dpy, screen);
    let mut drw = drw::Drw::new(dpy, screen, root, sw, sh);
    // Temporary. TODO : replace with real parameter
    let mut fonts = Vec::new();
    fonts.push("Fixed:size=9");
    // 
    drw.load_fonts(fonts);
    if drw.fontcount<1 {
        eprintln!("no fonts could be loaded.\n");
        process::exit(1);
    }
    drw
}

/// Main loop
unsafe fn run(dpy: *mut xlib::Display) {
    let ev: *mut xlib::XEvent = ptr::null_mut();
    xlib::XSync(dpy, 0);
    while running && xlib::XNextEvent(dpy, ev)==0 {
        // TODO les handlers d'events
        println!("Event : {:?}", ev);
    }
}

fn quit(_: Arg) {
    unsafe { running = false };
}
