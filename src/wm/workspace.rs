extern crate x11;

use std::collections::LinkedList;

use x11::xlib;

use { Client, Pertag };
use { SCHEMENORM, SCHEMESEL };
use drw;
use drw::Drw;
use drw::clrscheme::ClrScheme;
use config;

/**
 * Stores a layout
 */
pub struct Layout<'a> {
    pub symbol: &'a str,
    pub arrange: fn(&Workspace)
}

/**
 * Stores a monitor
 */
pub struct Workspace<'a> {
    pub ltsymbol: &'a str,
    pub mfact: f32,
    pub nmaster: u32,
    pub num: i32,
    pub tag: &'a str,
    pub by: i32,    // Y position of Bar
    pub x: i32, pub y: i32, pub w: u32, pub h: u32, // Workspace
    pub seltags: u32,
    pub sellt: u32,
    pub tagset: Vec<u32>,
    pub showbar: bool,
    pub topbar: bool,
    pub clients: LinkedList<Client<'a>>,
    pub barwin: xlib::Window,
    pub lt: Vec<&'a Layout<'a>>,
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
        ltsymbol: config::layouts[0].symbol.clone(),
        mfact: config::mfact,
        nmaster: config::nmaster,
        num: 0,
        tag,
        by: 0,
        x: 0, y: 0, w: 0, h: 0,
        seltags: 0,
        sellt: 0,
        tagset: Vec::new(),
        showbar: config::showbar,
        topbar: config::topbar,
        clients: LinkedList::new(),
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


/**
 * Updates the position of the statusbar for this Workspace
 */
pub fn updateBarPos(ws : Workspace, bh: u32) -> Workspace {
    if ws.showbar {
        return Workspace {
            by: if ws.topbar { ws.y } else { (ws.h as i32) - (bh as i32)},
            ..ws
        };
    }
    Workspace {
        by: -(bh as i32),
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
    let (drw, _) = config::tags.iter().enumerate().fold((drw, 0), |(drw, x), (i, t)| {
        let w = drw::textw(t, &drw);
        let (drw, _) = drw::text(if i == selmonindex { drw::setScheme(drw, &scheme[SCHEMESEL]) }
                                 else { drw::setScheme(drw, &scheme[SCHEMENORM]) },
                                 x, 1, w, bh, t, urg & (1 << i) != 0);
        let drw = if wss[0].clients.len() > 0 {
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
