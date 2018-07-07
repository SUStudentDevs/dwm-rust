extern crate x11;

use std::collections::LinkedList;

use x11::xlib;

use { Client, Pertag };
use { SCHEMENORM, SCHEMESEL };
use drw::Drw;
use drw::clrscheme::ClrScheme;
use config;

/**
 * Width of a text
 */
pub fn textw(s: &str, drw: &mut Drw) -> u32 {
    // drw.text(0, 0, 0, 0, s, false) as u32 +
        drw.fonts[0].h
}

/**
 * Area of a rectangle
 */
pub fn intersect(x: i32, y: i32, w: u32, h: u32, m: &Monitor) -> u32 {
    0.max((x as u32 + w).min(m.wx as u32 + m.ww - x.max(m.wx) as u32)) *
    0.max((y as u32 + h).min(m.wy as u32 + m.wh - y.max(m.wy) as u32))
}

/**
 * Stores a layout
 */
pub struct Layout<'a> {
    pub symbol: &'a str,
    pub arrange: fn(&Monitor)
}

/**
 * Stores a monitor
 */
pub struct Monitor<'a> {
    pub ltsymbol: &'a str,
    pub mfact: f32,
    pub nmaster: u32,
    pub num: i32,
    pub by: i32,    // Bar
    pub mx: i32, pub my: i32, pub mw: u32, pub mh: u32, // Monitor
    pub wx: i32, pub wy: i32, pub ww: u32, pub wh: u32, // Window
    pub seltags: u32,
    pub sellt: u32,
    pub tagset: Vec<u32>,
    pub showbar: bool,
    pub topbar: bool,
    pub clients: LinkedList<Client<'a>>,
    pub sel: Option<&'a Client<'a>>,
    pub stack: LinkedList<&'a Client<'a>>,
    pub barwin: xlib::Window,
    pub lt: Vec<&'a Layout<'a>>,
    pub pertag: Pertag<'a>
}

impl<'a> PartialEq for Monitor<'a> {
    fn eq(&self, other: &Monitor<'a>) -> bool {
        self.num == other.num
    }
}

/**
 * Creates a new monitor
 */
pub fn createMonitor<'a>() -> Monitor<'a> {
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

// /**
//  * Finds the Monitor intersecting with a rectangle
//  */
// pub fn from_rect(x: i32, y: i32, w: u32, h: u32, mons: &'a Vec<Monitor<'a>>, selmon: &'a Monitor<'a>) -> &'a Monitor<'a> {
//     let mut area = 0;
//     let mut r = selmon;
//     for m in mons.into_iter() {
//         let a = intersect(x, y, w, h, m);
//         if a > area {
//             area = a;
//             r = m;
//         }
//     }
//     r
// }

// /**
//  * Finds the Monitor a Window is on
//  */
// pub fn from_window(w: xlib::Window, root: xlib::Window, mons: &'a Vec<Monitor<'a>>, selmon: &'a Monitor<'a>) -> &'a Monitor<'a> {
//     if w == root && true /* TODO */ {
//         return Monitor::from_rect(0, 0, 1, 1, mons, selmon);
//     }
//     // TODO
//     if let Some(c) = Client::from(w, mons) {
//         return &mons[c.monindex];
//     }
//     selmon
// }

// /**
//  * Draws the statusbar for this Monitor
//  */
// pub fn drawbar(&self, drw: &mut Drw, bh: u32, scheme: &mut Vec<ClrScheme>, selmon: &Monitor<'a>, stext: &str) {
//     let dx: u32 = ((drw.fonts[0].ascent + drw.fonts[0].descent + 2) / 4) as u32;
//     let mut occ = 0;
//     let mut urg = 0;
//     for mut c in self.clients.iter() {
//         occ = occ|c.tags;
//         if c.isurgent {
//             urg = urg|c.tags
//         }
//     }

//     // Draw list of monitors, with their tags
//     let mut x = 0;
//     for i in 0..config::tags.len() {
//         let w = textw(config::tags[i], drw);
//         if self.tagset[self.seltags as usize] & 1 << i != 0 { // crashes because of self. why ?
//             drw.setscheme(&mut scheme[SCHEMESEL]);
//         } else {
//             drw.setscheme(&mut scheme[SCHEMENORM]);
//         }
//         drw.text(x, 1, w, bh, config::tags[i], urg & (1 << i) != 0);
//         if let Some(sel) = selmon.sel {
//             drw.rect(x + 1, 1, dx, dx, self == selmon && sel.tags & (1 << i) != 0, occ & (1 << i) != 0, urg & (1 << i) != 0);
//         } else {
//             drw.rect(x + 1, 1, dx, dx, false, occ & (1 << i) != 0, urg & (1 << i) != 0);
//         }
//         x += w as i32;
//     }

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
//     drw.map(self.barwin, 0, 0, self.ww, bh); // C'est la que ca crashe : self.ww = 0 ?
// }

// /**
//  * Updates the position of the statusbar for this Monitor
//  */
// pub fn updatebarpos(&mut self, bh: u32) {
//     self.wy = self.my;
//     self.wh = self.mh;
//     if self.showbar {
//         self.wh -= bh;
//         if self.topbar {
//             self.by = self.wy; self.wy = self.wy + bh as i32;
//         } else {
//             self.by = self.wy + self.wh as i32;
//         }
//     } else {
//         self.by = -(bh as i32);
//     }
// }
