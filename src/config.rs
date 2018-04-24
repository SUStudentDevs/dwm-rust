use x11::xlib;
use x11::keysym;

use { Layout, Key, Arg };
use { tilearrange, monoclearrange, noarrange, gridarrange };
use { quit };

/// Appearance
pub const fonts: [&str; 1] = ["Fixed:size=9"];

pub const normbordercolor: &str = "#444444";
pub const normbgcolor: &str = "#222222";
pub const normfgcolor: &str = "#bbbbbb";
pub const selbordercolor: &str = "#005577";
pub const selbgcolor: &str = "#005577";
pub const selfgcolor: &str = "#eeeeee";
pub const borderpx: u32 = 2;    // Border pixel of windows
pub const snap: u32 = 32;       // Snap pixel
pub const showbar: bool = true;     // false means no bar
pub const topbar: bool = false;      // false means bottombar

/// Layouts
pub const mfact: f32 = 0.5;     // Ratio of master area width
pub const nmaster: u32 = 2;     // Number of clients in master area

pub const layouts: [Layout; 4] = [
    Layout { symbol: "[]=", arrange: tilearrange },
    Layout { symbol: "[M]", arrange: monoclearrange },
    Layout { symbol: "><>", arrange: noarrange },
    Layout { symbol: "HHH", arrange: gridarrange }
];

pub const tags: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Keys and buttons
pub const MODKEY: u32 = xlib::Mod4Mask;

pub const keys: [Key; 1] = [
    //    modifier              key                 function                argument
    // Key { modif:MODKEY|xlib::ShiftMask, keysym:keysym::XK_e as u64, func:quit, arg:Arg {i: 0}},
    Key { modif:MODKEY, keysym:keysym::XK_e as u64, func:quit, arg:Arg {i: 0}},
];
