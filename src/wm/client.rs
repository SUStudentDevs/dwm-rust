extern crate x11;

use x11::xlib;

use wm::monitor::Monitor;

/**
 * Stores a Client (wrapper around the xlib::Window struct)
 */
pub struct Client<'a> {
    pub name: String,
    pub mina: f32, pub maxa: f32,
    pub x: i32, pub y: i32, pub w: i32, pub h: i32,
    pub oldx: i32, pub oldy: i32, pub oldw: i32, pub oldh: i32,
    pub basew: i32, pub baseh: i32, pub incw: i32, pub inch: i32, pub maxw: i32, pub maxh: i32, pub minw: i32, pub minh: i32,
    pub bw: i32, pub oldbw: i32,
    pub tags: u32,
    pub isfixed: bool, pub isfloating: bool, pub isurgent: bool, pub neverfocus: bool, pub oldwm:bool, pub isfullscreen: bool,
    pub mon: &'a mut Monitor<'a>,
    pub win: xlib::Window
}

impl<'a> Client<'a> {
    /**
     * Finds the Client containing a Window
     */
    pub fn from(window : xlib::Window, mons: &'a mut Vec<Monitor<'static>>) -> Option<&'a mut Client<'static>> {
        for m in mons.iter_mut() {
            for c in m.clients.iter_mut() {
                if c.win == window {
                    return Some(c)
                }
            }
        }
        None
    }
}
