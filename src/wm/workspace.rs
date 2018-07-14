extern crate x11;

use std::collections::LinkedList;

use x11::xlib;

use client;
use { Client, Pertag };
use { SCHEMENORM, SCHEMESEL };
use drw;
use drw::Drw;
use drw::clrscheme::ClrScheme;
use config;

/// Arrange functions
pub fn tileArrange(mut ws: Workspace) -> Workspace {
    let n = ws.clients.len() as u32;
    let x = minX(&ws); let y = minY(&ws); let w = maxW(&ws); let h = maxH(&ws);
    if n == 1 { // If there is only one window
        Workspace {
            clients: vec! [client::setGeom(ws.clients.remove(0), x, y, w, h)],
            ..ws
        }
    } else if n > 1 {
        let w = w/n;
        Workspace {
            clients: ws.clients.into_iter().enumerate().map(|(i, c)| { client::setGeom(c, x+(i as i32 * w as i32), y, w, h) }).collect(),
            ..ws
        }
    } else {
        ws
    }
}

pub fn monocleArrange(ws: Workspace) -> Workspace {
    // TODO
    ws
}

pub fn noArrange(ws: Workspace) -> Workspace {
    ws // Nothing
}

pub fn gridArrange(ws: Workspace) -> Workspace {
    // TODO
    ws
}

/**
 * Stores a layout
 */
pub struct Layout<'a> {
    pub symbol: &'a str,
    pub arrange: fn (Workspace) -> Workspace
}

/**
 * Stores a monitor
 */
pub struct Workspace<'a> {
    pub mfact: f32,
    pub nmaster: u32,
    pub num: i32,
    pub tag: &'a str,
    pub by: i32, pub bh: u32,  // Y position and height of bar
    pub x: i32, pub y: i32, pub w: u32, pub h: u32, // Workspace
    pub seltags: u32,
    pub sellt: u32,
    pub tagset: Vec<u32>,
    pub showbar: bool,
    pub topbar: bool,
    pub clients: Vec<Client<'a>>,
    pub barwin: xlib::Window,
    pub lt: Layout<'a>,
    pub pertag: Pertag<'a>
}

impl<'a> PartialEq for Workspace<'a> {
    fn eq(&self, other: &Workspace<'a>) -> bool {
        self.num == other.num
    }
}

/**
 * Creates a new monitor
 */
pub fn createWorkspace<'a>(tag: &'a str) -> Workspace<'a> {
    let mut mon = Workspace {
        mfact: config::mfact,
        nmaster: config::nmaster,
        num: 0,
        tag,
        by: 0, bh: 0,
        x: 0, y: 0, w: 0, h: 0,
        seltags: 0,
        sellt: 0,
        tagset: Vec::new(),
        showbar: config::showbar,
        topbar: config::topbar,
        clients: Vec::new(),
        barwin: 0,
        lt: Layout { symbol: &config::layouts[0].symbol, arrange: config::layouts[0].arrange },
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
    mon
    // TODO tags
}

// /**
//  * Finds the Workspace a Window is on
//  */
// pub fn from_window(w: xlib::Window, root: xlib::Window, mons: &'a Vec<Workspace<'a>>, selmon: &'a Workspace<'a>) -> &'a Workspace<'a> {
//     if w == root && true /* TODO */ {
//         return Workspace::from_rect(0, 0, 1, 1, mons, selmon);
//     }
//     // TODO
//     if let Some(c) = Client::from(w, mons) {
//         return &mons[c.monindex];
//     }
//     selmon
// }

pub fn minX(ws: &Workspace) -> i32 { ws.x }

pub fn maxW(ws: &Workspace) -> u32 { ws.w }

pub fn minY(ws : &Workspace) -> i32 {
    if ws.showbar && ws.topbar { ws.bh as i32} else { ws.y }
}

pub fn maxH(ws: &Workspace) -> u32 {
    let m = if ws.showbar { ws.h - ws.bh as u32 } else { ws.h };
    m
}

/**
 * Updates the position of the statusbar for this Workspace
 */
pub fn updateBarPos(ws: Workspace, bh: u32) -> Workspace {
    if ws.showbar {
        return Workspace {
            by: if ws.topbar { ws.y } else { (ws.h as i32) - (bh as i32)},
            bh,
            ..ws
        };
    }
    Workspace {
        by: -(bh as i32),
        bh,
        ..ws
    }
}

/**
 * Draws the statusbar
 */
pub fn drawBar<'a>(drw: Drw<'a>, bh: u32, scheme: &Vec<ClrScheme>, wss: &Vec<Workspace>, selmonindex: usize, stext: &str) -> Drw<'a> {
    let w = drw.w;
    let drw = drw::rect(drw::setScheme(drw, &scheme[SCHEMENORM]), 0, 0, w, bh, true, true);
    let dx: u32 = ((drw.fonts[0].ascent + drw.fonts[0].descent + 2) / 4) as u32;
    let occ = 0;
    let urg = 0;
//     for mut c in self.clients.iter() {
//         occ = occ|c.tags;
//         if c.isurgent {
//             urg = urg|c.tags
//         }
//     }

    // Draw list of workspaces, with their tags
    let (drw, _) = wss.iter().enumerate().fold((drw, 0), |(drw, x), (i, ws)| {
        let (drw, w) = drw::textw(ws.tag, drw);
        let (drw, _) = drw::text(if i == selmonindex { drw::setScheme(drw, &scheme[SCHEMESEL]) }
                                 else { drw::setScheme(drw, &scheme[SCHEMENORM]) },
                                 x, 1, w, bh, ws.tag, urg & (1 << i) != 0);
        let drw = if ws.clients.len() > 0 {
            drw::rect(drw, x + 1, 1, dx, dx, i == selmonindex, occ & (1 << i) != 0)
        }
        else { drw };
        (drw, x + w as i32)
    });
    // for i in 0..config::tags.len() {
    //     let w = textw(config::tags[i], &drw);
    //     // let drw = if drw.tagset[drw.seltags as usize] & 1 << i != 0 {
    //     //     drw::setScheme(drw, &mut scheme[SCHEMESEL]);
    //     // } else {
    //     //     drw::setScheme(drw, &mut scheme[SCHEMENORM]);
    //     // };
    // }

//     let blw = textw(self.ltsymbol, drw);
//     let mut w = blw;
//     drw.setscheme(&mut scheme[SCHEMENORM]);
//     drw.text(x, 0, w, bh, self.ltsymbol, false);
//     x += w as i32;
//     let xx = x;
//     if self == selmon { // Status is only drawn on selected monitor
//         w = textw(stext, drw);
//         x = self.ww as i32 - w as i32;
//         if x < xx {
//             x = xx;
//             w = self.ww - xx as u32;
//         }
//         drw.text(x, 0, w, bh, stext, false);
//     } else {
//         x = self.ww as i32;
//     }
//     w = (x - xx) as u32;
//     if w > bh {
//         x = xx;
//         if let Some(sel) = self.sel {
//             if self == selmon {
//                 drw.setscheme(&mut scheme[SCHEMESEL]);
//             } else {
//                 drw.setscheme(&mut scheme[SCHEMENORM]);
//             }
//             drw.text(x, 0, w, bh, &sel.name[..], false);
//             drw.rect(x + 1, 1, dx, dx, sel.isfixed, sel.isfloating, false);
//         } else {
//             drw.setscheme(&mut scheme[SCHEMENORM]);
//             drw.rect(x, 0, w, bh, true, false, true);
//         }
//     }
    let w = drw.w;
    drw::mapWindow(drw, wss[selmonindex].barwin, 0, 0, w, bh) // C'est la que ca crashe : self.ww = 0 ?
}

/**
 * Adds a Client to this Workspace
 */
pub fn addClient<'a>(ws: &'a mut Workspace<'a>, c: Client<'a>) {
    ws.clients.insert(0, c);
}

/**
 * Removes a Client from this Workspace, returning it
 */
pub fn removeClient<'a>(ws: &mut Workspace<'a>, c: &Client<'a>) -> Option<Client<'a>> {
    for i in 0..ws.clients.len() {
        if &ws.clients[i] == c {
            let cl = ws.clients.remove(i);
            return Some(cl);
        }
    }
    None
}

/**
 * Updates geometry of the Workspace
 */
pub fn updateGeom<'a>(ws: Workspace<'a>, dpy: &mut xlib::Display) -> Workspace<'a> {
    let arrange = ws.lt.arrange;
    let ws = arrange(ws);
    for c in ws.clients.iter() {
        client::configure(c, dpy);
    }
    ws
}

/**
 * Draws all the windows in this workspace
 */
pub fn showAllClients(ws: &Workspace, dpy: &mut xlib::Display) {
    for c in ws.clients.iter() { client::show(c, dpy); }
}

/**
 * Draws all the windows in this workspace
 */
pub fn hideAllClients(ws: &Workspace, dpy: &mut xlib::Display) {
    for c in ws.clients.iter() { client::hide(c, dpy); }
}
