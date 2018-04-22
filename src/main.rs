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

// Fn instead of C macros
fn textw(s: &str, drw: &mut drw::Drw) -> u32 {
    drw.text(0, 0, 0, 0, s, false) as u32 + drw.fonts[0].h
}

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
    isfixed: bool, isfloating: bool, isurgent: bool, neverfocus: bool, oldwm:bool, isfullscreen: bool,
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
    mx: i32, my: i32, mw: u32, mh: u32, // Monitor
    wx: i32, wy: i32, ww: u32, wh: u32, // Window
    seltags: u32,
    sellt: u32,
    tagset: Vec<u32>,
    showbar: bool,
    topbar: bool,
    clients: LinkedList<Client<'a>>,
    sel: Option<&'a Client<'a>>,
    stack: LinkedList<&'a Client<'a>>,
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
            clients: LinkedList::new(),
            sel: None,
            stack: LinkedList::new(),
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

impl<'a> PartialEq for Monitor<'a> {
    fn eq(&self, other: &Monitor<'a>) -> bool {
        self.num == other.num
    }
}

struct WM<'a> {
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
    selmon: *mut Monitor<'a>,
    sw: u32, sh: u32,
    bh: u32,
    stext: String
}

impl<'a> WM<'a> {
    fn new(dpy: &mut xlib::Display, drw: drw::Drw, screen: i32, root: u64, sw: u32, sh: u32) -> WM {
        let mut wm = WM {
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
            selmon: ptr::null_mut(),
            sw, sh,
            bh: 0, 
            stext: String::from("dwm-rust")
        }; 
        wm.bh = wm.drw.fonts[0].h + 2; 
        unsafe {
            // Init atoms
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_PROTOCOLS").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_DELETE_WINDOW").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_STATE").unwrap().as_ptr(), 0));
            wm.wmatom.push(xlib::XInternAtom(wm.dpy, CString::new("WM_TAKE_FOCUS").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy,CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_SUPPORTED").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_NAME").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_STATE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_WINDOWN_TYPE").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_WM_WINDOW_TYPE_DIALOG").unwrap().as_ptr(), 0));
            wm.netatom.push(xlib::XInternAtom(wm.dpy, CString::new("_NET_CLIENT_LIST").unwrap().as_ptr(), 0));
            // Init cursors
            wm.cursor.push(drw::Cur::new(&mut (wm.drw), 68)); // Normal
            wm.cursor.push(drw::Cur::new(&mut (wm.drw), 120)); // Resize
            wm.cursor.push(drw::Cur::new(&mut (wm.drw), 52)); // Move
            // Init color schemes
            wm.scheme.push(drw::ClrScheme::new(
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::normfgcolor),
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::normbgcolor),
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::normbordercolor))); // Normal
            wm.scheme.push(drw::ClrScheme::new(
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::selfgcolor),
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::selbgcolor),
                    drw::Clr::new(wm.dpy, wm.drw.screen, config::selbordercolor))); // Selected
            } 
        wm 
    } 

    /// Updates the geometry
    fn updategeom(&mut self) -> bool {
        let mut dirty = false;
        if unsafe { xinerama::XineramaIsActive(self.dpy) }!=0 {
            // TODO
        } else {
            if self.mons.is_empty() {
                self.mons.push_front(Monitor::new());
            } 
            if let Some(mut m) = self.mons.front_mut() {
                if m.ww != self.sw {
                    dirty = true;
                    m.mw = self.sw; m.ww = self.sw;
                    m.mh = self.sh; m.wh = self.sh;
                    updatebarpos(&mut m, self.bh);
                }
            }
        }
        if dirty {
            // TODO
            if let Some(mon) = self.mons.front_mut() {
                self.selmon = mon; // TODO
            }
            //wm.selmon = Some(wintomon(wm, wm.root));
        }
        dirty
    }

    /// Updates the status bars
    fn updatebars(&mut self) {
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
                    xlib::XCreateWindow(self.dpy,
                                        self.root,
                                        m.wx, m.by, m.ww as u32, 
                                        self.bh, 
                                        0, 
                                        xlib::XDefaultDepth(self.dpy, self.screen), 
                                        xlib::CopyFromParent as u32, 
                                        xlib::XDefaultVisual(self.dpy, self.screen), 
                                        xlib::CWOverrideRedirect|xlib::CWBackPixmap|xlib::CWEventMask, &mut wa) };
                unsafe { xlib::XDefineCursor(self.dpy, m.barwin, self.cursor[CURNORMAL].cursor) };
                unsafe { xlib::XMapRaised(self.dpy, m.barwin) };
            }
        }
    }

    

    fn updatestatus(&mut self) {
        // if(...) TODO
        drawbar(self.selmon, &mut (self.drw), self.bh, &mut self.scheme, self.selmon, &self.stext[..]);
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
    }
    if unsafe { xlib::XSupportsLocale() } == 0 {
        println!("Warning : no locale support");
    } if let Some(dpy) = Some( unsafe { &mut(*xlib::XOpenDisplay(ptr::null())) }) {
        // This is where we'll work
        checkotherwm(dpy);
        let mut wm = setup(dpy);
        run(&mut wm); 
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
    let mut drw = drw::Drw::new(dpy, screen, root, sw, sh);

    drw.load_fonts(config::fonts.to_vec());
    if drw.fontcount<1 {
        eprintln!("no fonts could be loaded.\n");
        process::exit(1);
    }

    let mut wm : WM = WM::new(dpy, drw, screen, root, sw, sh);
    // Init bars
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
    // grabkeys(); TODO
    // focus(None); TODO
    wm
}

/// Update position of a bar
fn updatebarpos(m: &mut Monitor, bh: u32) {
    m.wy = m.my;
    m.wh = m.mh;
    if m.showbar {
        m.wh -= bh;
        if m.topbar {
            m.by = m.wy; m.wy = m.wy + bh as i32;
        } else {
            m.by = m.wy + m.wh as i32;
        }
    } else {
        m.by = -(bh as i32);
    }
}

fn wintomon<'a>(wm: &'a mut WM<'a>, w: xlib::Window) -> &'a mut Monitor<'a> {
    // TODO
    if w == wm.root {
        // TODO recttomon

    }
    wm.mons.front_mut().unwrap()
}

fn drawbar<'a>(m: *mut Monitor<'a>, drw: &mut drw::Drw, bh: u32, scheme: &mut Vec<drw::ClrScheme>, selmon: *mut Monitor<'a>, stext: &str) {
    let dx: u32 = ((drw.fonts[0].ascent + drw.fonts[0].descent + 2) / 4) as u32;
    let mut occ = 0;
    let mut urg = 0;
    unsafe {
        for mut c in (*m).clients.iter() {
            occ = occ|c.tags;
            if c.isurgent {
                urg = urg|c.tags
            }
        }

        // Draw list of monitors, with their tags
        let mut x = 0;
        for i in 0..config::tags.len() {
            let w: u32 = textw(config::tags[i], drw);
            if (*m).tagset[(*m).seltags as usize] & 1 << i != 0 {
                drw.setscheme(&mut scheme[SCHEMESEL]);
            } else {
                drw.setscheme(&mut scheme[SCHEMENORM]);
            }
            drw.text(x, 1, w, bh, config::tags[i], urg & (1 << i) != 0);
            if let Some(sel) = (*selmon).sel {
                drw.rect(x + 1, 1, dx, dx, (*m).eq(&(*selmon)) && sel.tags & (1 << i) != 0, occ & (1 << i) != 0, urg & (1 << i) != 0);
            } else {
                drw.rect(x + 1, 1, dx, dx, false, occ & (1 << i) != 0, urg & (1 << i) != 0);
            }
            x += w as i32;
        }
        let blw = textw((*m).ltsymbol, drw);
        let mut w = blw;
        drw.setscheme(&mut scheme[SCHEMENORM]);
        drw.text(x, 0, w, bh, (*m).ltsymbol, false);
        x += w as i32;
        let xx = x;
        if m == selmon { // Status is only drawn on selected monitor
            w = textw(stext, drw);
            x = (*m).ww as i32 - w as i32;
            if x < xx {
                x = xx;
                w = (*m).ww - xx as u32;
            }
            drw.text(x, 0, w, bh, stext, false);
        } else {
            x = (*m).ww as i32;
        }
        w = (x - xx) as u32;
        if w > bh {
            x = xx;
            if let Some(sel) = (*m).sel {
                if m == selmon {
                    drw.setscheme(&mut scheme[SCHEMESEL]);
                } else {
                    drw.setscheme(&mut scheme[SCHEMENORM]);
                }
                drw.text(x, 0, w, bh, &sel.name[..], false);
                drw.rect(x + 1, 1, dx, dx, sel.isfixed, sel.isfloating, false);
            } else {
                drw.setscheme(&mut scheme[SCHEMENORM]);
                drw.rect(x, 0, w, bh, true, false, true);
            }
        }
        drw.map((*m).barwin, 0, 0, (*m).ww, bh);
    }
}

/// Main loop
fn run(wm: &mut WM) {
    let ev = &mut xlib::XEvent { any: xlib::XAnyEvent { type_: 0, serial: 0, send_event: 0, display: wm.dpy, window: wm.root } }; // Dummy value
    unsafe {
        xlib::XSync(wm.dpy, 0); // Ca crashe la
        while wm.running && xlib::XNextEvent(wm.dpy, ev)==0 {
            handleevent(wm.dpy, ev);
        }
    }
}

/// Handle an event
fn handleevent(dpy: &mut xlib::Display, ev: *mut xlib::XEvent) {
    unsafe {
        match (*ev).type_ {
            xlib::ButtonPress => buttonpress(dpy),
            // TODO : les autres handlers
            _ => ()
        }
    }
}

/// Handle a button press
fn buttonpress(dpy: &mut xlib::Display) {
    // TODO
    println!("boutton"); // TEMPORARY
}

/// Quit the wm
fn quit(_: Arg, wm: &mut WM) {
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
