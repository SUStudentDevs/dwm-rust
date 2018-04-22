extern crate x11;

use x11::xlib;

use wm::monitor::Monitor;

/// Client (Window) class
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
