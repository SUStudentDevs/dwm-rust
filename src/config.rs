use x11::keysym::*;
use x11::xlib;

use wm::workspace::{gridArrange, monocleArrange, noArrange, tileArrange};
use {changeWs, closeClient, moveClientToWs, quit, spawn};
use {Arg, Button, Key, Layout};

/// Fonts (the first one available is used)
pub const fonts: [&str; 1] = ["Fixed:size=11"];

pub const normbordercolor: &str = "#444444";
pub const normbgcolor: &str = "#222222";
pub const normfgcolor: &str = "#bbbbbb";
pub const selbordercolor: &str = "#005577";
pub const selbgcolor: &str = "#005577";
pub const selfgcolor: &str = "#eeeeee";
/// Background color
pub const backgroundColor: u64 = 0x00aa00;
/// Size (in pixels) of window borders
pub const borderpx: u32 = 2;
/// Snap pixel
pub const snap: u32 = 32;
/// Show the status bar (false means no bar)
pub const showbar: bool = true;
/// Show the status bar on top (false means bottom)
pub const topbar: bool = true;
// Bar time formatting
pub const timeFormat: &str = "%H:%M:%S - %d %b %Y";

/// Ratio of master area to stack area width
pub const mfact: f32 = 0.5;
/// Maximum number of clients in the master area
pub const nmaster: u32 = 1;

/// Layouts
pub const layouts: [Layout; 4] = [
    Layout {
        symbol: "[]=",
        arrange: tileArrange,
    },
    Layout {
        symbol: "[M]",
        arrange: monocleArrange,
    },
    Layout {
        symbol: "><>",
        arrange: noArrange,
    },
    Layout {
        symbol: "HHH",
        arrange: gridArrange,
    },
];

/// Tags
pub const tags: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Modifier key for key controls; Mod4Mask by default
pub const MODKEY: u32 = xlib::Mod4Mask;

/// Key combinations and their actions
pub const keys: [Key; 25] = [
    //    modifier              key                 function                argument
    Key {
        modif: MODKEY,
        keysym: XK_Return as u64,
        func: spawn,
        arg: Arg { s: "terminator" },
    },
    Key {
        modif: MODKEY,
        keysym: XK_d as u64,
        func: spawn,
        arg: Arg {
            s: "rofi -show run",
        },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_e as u64,
        func: quit,
        arg: Arg { i: 0 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_q as u64,
        func: closeClient,
        arg: Arg { i: 0 },
    },
    // Change WS
    Key {
        modif: MODKEY,
        keysym: XK_1 as u64,
        func: changeWs,
        arg: Arg { u: 1 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_2 as u64,
        func: changeWs,
        arg: Arg { u: 2 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_3 as u64,
        func: changeWs,
        arg: Arg { u: 3 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_4 as u64,
        func: changeWs,
        arg: Arg { u: 4 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_5 as u64,
        func: changeWs,
        arg: Arg { u: 5 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_6 as u64,
        func: changeWs,
        arg: Arg { u: 6 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_7 as u64,
        func: changeWs,
        arg: Arg { u: 7 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_8 as u64,
        func: changeWs,
        arg: Arg { u: 8 },
    },
    Key {
        modif: MODKEY,
        keysym: XK_9 as u64,
        func: changeWs,
        arg: Arg { u: 9 },
    },
    // Move window to WS
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_1 as u64,
        func: moveClientToWs,
        arg: Arg { u: 1 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_2 as u64,
        func: moveClientToWs,
        arg: Arg { u: 2 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_3 as u64,
        func: moveClientToWs,
        arg: Arg { u: 3 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_4 as u64,
        func: moveClientToWs,
        arg: Arg { u: 4 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_5 as u64,
        func: moveClientToWs,
        arg: Arg { u: 5 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_6 as u64,
        func: moveClientToWs,
        arg: Arg { u: 6 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_7 as u64,
        func: moveClientToWs,
        arg: Arg { u: 7 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_8 as u64,
        func: moveClientToWs,
        arg: Arg { u: 8 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XK_9 as u64,
        func: moveClientToWs,
        arg: Arg { u: 9 },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XF86XK_AudioLowerVolume as u64,
        func: spawn,
        arg: Arg {
            s: "amixer -q sset 'Master' 5%-",
        },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XF86XK_AudioRaiseVolume as u64,
        func: spawn,
        arg: Arg {
            s: "amixer -q sset 'Master' 5%+",
        },
    },
    Key {
        modif: MODKEY | xlib::ShiftMask,
        keysym: XF86XK_AudioMute as u64,
        func: spawn,
        arg: Arg {
            s: "amixer -q sset 'Master' ",
        },
    },
];

/// Buttons and their actions
pub const buttons: [Button; 0] = [];

/// Commands to execute at start of the wm
pub const startCmds: [&str; 2] = [
    "feh --bg-scale /home/vertmo/Images/Wallpapers/botw.png",
    "statusbar",
];
