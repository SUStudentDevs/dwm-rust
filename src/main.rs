extern crate x11;

pub mod config;
pub mod drw;

use std::env;
use std::process;
use std::ptr;
use std::collections::LinkedList;
use std::ffi::CString;

use x11::xlib;
use x11::xinerama;

const VERSION: &str = "0.0.1";

// WM Atom indexes
const WMPROTOCOLS: usize = 0; const WMDELETE: usize = 1; const WMSTATE: usize  = 2; const WMTAKEFOCUS: usize = 3; const WMLAST: usize = 4;

// Net Atom indexes
const NETACTIVEWINDOW: usize = 0; const NETSUPPORTED: usize = 1; const NETWMNAME: usize = 2; const NETWMSTATE: usize = 3; const NETWMFULLSCREEN: usize = 4; const NEWWMWINDOWTYPE: usize = 5; const NETWMWINDOWTYPEDIALOG: usize = 6; const NETCLIENTLIST: usize = 7; const NETLAST: usize = 8;

// Cursor indexes
const CURNORMAL: usize = 0; const CURRESIZE: usize = 1; const CURMOVE: usize = 2;

// Color scheme indexes
const SCHEMENORM: usize = 0; const SCHEMESEL: usize = 1;

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
    mon: &'a mut Monitor<'a>,
    win: xlib::Window
}

pub struct Layout<'a> {
    symbol: &'a str,
    arrange: fn(&Monitor)
}

struct Pertag<'a> {
    curtag: u32, prevtag: u32,  // Current and previous tag
    nmasters: Vec<i32>, // number windows in master area
    mfacts: Vec<f32>,   // mfacts per tag
    selltds: Vec<u32>,  // Selected layouts
    ltidxs: Vec<Vec<&'a Layout<'a>>>, // Matrix of tags and yaouts
    showbars: Vec<bool>,    // Display bar for each tag
    prefzooms: Vec<&'a Client<'a>> // Zoom information
}

struct Monitor<'a> {
    ltsymbol: &'a str,
    mfact: f32,
    nmaster: u32,
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
    barwin: xlib::Window,
    lt: Vec<&'a Layout<'a>>,
    pertag: Pertag<'a>
}

impl<'a> Monitor<'a> {
    fn new<'b>() -> Monitor<'a> {
        let mut mon = Monitor {
            ltsymbol: config::layouts[0].symbol.clone(),
            mfact: config::mfact,
            nmaster: config::nmaster,
            num: 0,
            by: 0,
            mx: 0, my: 0, mw: 0, mh: 0,
            wx: 0, wy: 0, ww: 0, wh: 0,
            seltags: 0,
            sellt: 0,
            tagset: Vec::new(),
            showbar: config::showbar,
            topbar: config::topbar,
            clients: None,
            sel: None,
            stack: None,
            barwin: 0,
            lt: Vec::new(),
            pertag: Pertag {
                curtag: 1,
                prevtag: 1,
                nmasters: Vec::new(),
                mfacts: Vec::new(),
                selltds: Vec::new(),
                ltidxs: Vec::new(),
                showbars: Vec::new(),
                prefzooms: Vec::new()
            }
        };
        mon.tagset.push(1); mon.tagset.push(1);
        mon.lt.push(&config::layouts[0]); 
        mon.lt.push(&config::layouts[1 % config::layouts.len()]);
        mon
        // TODO tags
    }
}

struct State<'a> {
    dpy: &'a mut xlib::Display,
    drw: drw::Drw,
    screen: i32,
    root: u64,
    running: bool,
    wmatom: Vec<xlib::Atom>,
    netatom: Vec<xlib::Atom>,
    cursor: Vec<drw::Cur>,
    scheme: Vec<drw::ClrScheme>,
    mons: LinkedList<Monitor<'a>>,
    bh: u32
}

impl<'a> State<'a> {
    fn new<'b>(dpy: &'b mut xlib::Display, drw: drw::Drw, screen: i32, root: u64) -> State<'b> {
        let mut state = State {
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
            bh: 0,
        };
        state.bh = state.drw.fonts[0].h + 2;
        unsafe {
            // Init atoms
            state.wmatom.push(xlib::XInternAtom(state.dpy, CString::new("WM_PROTOCOLS").unwrap().as_ptr(), 0));
            state.wmatom.push(xlib::XInternAtom(state.dpy, CString::new("WM_DELETE_WINDOW").unwrap().as_ptr(), 0));
            state.wmatom.push(xlib::XInternAtom(state.dpy, CString::new("WM_STATE").unwrap().as_ptr(), 0));
            state.wmatom.push(xlib::XInternAtom(state.dpy, CString::new("WM_TAKE_FOCUS").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_SUPPORTED").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_WM_NAME").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_WM_STATE").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_WM_WINDOWN_TYPE").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_WM_WINDOW_TYPE_DIALOG").unwrap().as_ptr(), 0));
            state.netatom.push(xlib::XInternAtom(state.dpy, CString::new("_NET_CLIENT_LIST").unwrap().as_ptr(), 0));
            // Init cursors
            state.cursor.push(drw::Cur::new(&mut (state.drw), 68)); // Normal
            state.cursor.push(drw::Cur::new(&mut (state.drw), 120)); // Resize
            state.cursor.push(drw::Cur::new(&mut (state.drw), 52)); // Move
            // Init color schemes
            state.scheme.push(drw::ClrScheme::new(
                drw::Clr::new(state.dpy, state.drw.screen, config::normfgcolor),

                drw::Clr::new(state.dpy, state.drw.screen, config::normbgcolor),
                drw::Clr::new(state.dpy, state.drw.screen, config::normbordercolor))); // Normal
            state.scheme.push(drw::ClrScheme::new(
                drw::Clr::new(state.dpy, state.drw.screen, config::selfgcolor),
                drw::Clr::new(state.dpy, state.drw.screen, config::selbgcolor),
                drw::Clr::new(state.dpy, state.drw.screen, config::selbordercolor))); // Selected
        }
        state
    }
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
    } if let Some(dpy) = Some( unsafe { 
            &mut(*xlib::XOpenDisplay(ptr::null()))
    }) {
        // This is where we'll work
        checkotherwm(dpy);
        let mut state = setup(dpy);
        run(&mut state);
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
        eprintln!("dwm-rust: fatal error: request code={}, error code={}",
                  (*ee).request_code, (*ee).error_code);
        process::exit(1);
    }
}

/// Checks for another WM running. If there is one, prints an error and exits.
fn checkotherwm(dpy: *mut xlib::Display) {
    unsafe {
        xlib::XSetErrorHandler(Some(xerrorstart));
        xlib::XSelectInput(dpy, xlib::XDefaultRootWindow(dpy), xlib::SubstructureRedirectMask);
        xlib::XSync(dpy, 0);
        xlib::XSetErrorHandler(Some(xerror));
        xlib::XSync(dpy, 0);
    }
}

/// Setup everything
fn setup<'a>(dpy: &'a mut xlib::Display) -> State<'a> {
    unsafe {
        let screen = xlib::XDefaultScreen(dpy);
        let sw = xlib::XDisplayWidth(dpy, screen) as u32;
        let sh = xlib::XDisplayHeight(dpy, screen) as u32;
        let root = xlib::XRootWindow(dpy, screen);
        let mut drw = drw::Drw::new(dpy, screen, root, sw, sh);

        drw.load_fonts(config::fonts.to_vec());
        if drw.fontcount<1 {
            eprintln!("no fonts could be loaded.\n");
            process::exit(1);
        }

        let state = State::new(dpy, drw, screen, root);
        let state = updategeom(state);
        // Init bars TODO
        // let state = updatebars(state);
        // let state = updatestatus(state);
        xlib::XChangeProperty(state.dpy, state.root, state.netatom[NETSUPPORTED], xlib::XA_ATOM, 32, xlib::PropModeReplace, &(state.netatom[0] as u8), NETLAST as i32);
        xlib::XDeleteProperty(state.dpy, state.root, state.netatom[NETCLIENTLIST]); 
        xlib::XChangeWindowAttributes(state.dpy, state.root, xlib::CWEventMask|xlib::CWCursor, &mut xlib::XSetWindowAttributes {
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
            cursor: state.cursor[CURNORMAL].cursor
        });
        state
    }
}

/// Updates the geometry
fn updategeom<'a>(mut state: State<'a>) -> State {
    if unsafe { xinerama::XineramaIsActive(state.dpy) }!=0 {
        // TODO
        state
    } else {
        if state.mons.len()==0 {
            state.mons.push_back(Monitor::new());
            state
        } else {
            // TODO
            state
        }
    }
}

// Updates the status bars TODO : y'a une erreur qui vient de la
fn updatebars(mut state: State) -> State {
    let mut wa = xlib::XSetWindowAttributes {
        background_pixmap: xlib::ParentRelative as u64,
        background_pixel: 0, 
        border_pixmap: xlib::CopyFromParent as u64, 
        border_pixel: 0, 
        bit_gravity: xlib::ForgetGravity, 
        win_gravity: xlib::NorthWestGravity, 
        backing_store: xlib::NotUseful, 
        backing_planes: 1, 
        backing_pixel: 0, 
        save_under: 0,
        event_mask: xlib::ButtonPressMask|xlib::ExposureMask,
        do_not_propagate_mask: 0,
        override_redirect: 1,
        colormap: xlib::CopyFromParent as u64, 
        cursor: 0
    };
    for mut m in &mut state.mons {
        if m.barwin == 0 {
            m.barwin = unsafe { 
                xlib::XCreateWindow(state.dpy,
                                    state.root,
                                    m.wx, m.by, m.ww as u32, 
                                    state.bh, 
                                    0, 
                                    xlib::XDefaultDepth(state.dpy, state.screen), 
                                    xlib::CopyFromParent as u32, 
                                    xlib::XDefaultVisual(state.dpy, state.screen), 
                                    xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask, &mut wa) };
            unsafe { xlib::XDefineCursor(state.dpy, m.barwin, state.cursor[CURNORMAL].cursor) };
            unsafe { xlib::XMapRaised(state.dpy, m.barwin) };
        }
    }
    state
}

/// Main loop
fn run(state: &mut State) {
    let ev = &mut xlib::XEvent { any: xlib::XAnyEvent { type_: 0, serial: 0, send_event: 0, display: state.dpy, window: state.root } }; // Dummy value
    unsafe {
        println!("avant le XSync");
        xlib::XSync(state.dpy, 0); // Ca crashe la
        println!("apres le XSync");
        while state.running && xlib::XNextEvent(state.dpy, ev)==0 {
            println!("Event : {:?}", ev); // TEMPORARY
            handleevent(state.dpy, ev);
        }
    }
}

fn handleevent(dpy: &mut xlib::Display, ev: *mut xlib::XEvent) {
    unsafe {
        match (*ev).type_ {
            xlib::ButtonPress => buttonpress(dpy),
            // TODO : les autres handlers
            _ => ()
        }
    }
}

fn buttonpress(dpy: &mut xlib::Display) {
    // TODO
    println!("boutton"); // TEMPORARY
}

fn quit(_: Arg, state: &mut State) {
    state.running = false;
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
