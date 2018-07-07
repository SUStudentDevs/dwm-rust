use x11::xlib;
use x11::keysym::*;

use { Layout, Key, Button, Arg };
use { tilearrange, monoclearrange, noarrange, gridarrange };
use { spawn, quit };

/// Fonts (the first one available is used)
pub const fonts: [&str; 1] = ["Fixed:size=9"];

pub const normbordercolor: &str = "#444444";
pub const normbgcolor: &str = "#222222";
pub const normfgcolor: &str = "#bbbbbb";
pub const selbordercolor: &str = "#005577";
pub const selbgcolor: &str = "#005577";
pub const selfgcolor: &str = "#eeeeee";
/// Size (in pixels) of window borders
pub const borderpx: u32 = 2;
/// Snap pixel
pub const snap: u32 = 32;
/// Show the status bar (false means no bar)
pub const showbar: bool = true;
/// Show the status bar on top (false means bottom)
pub const topbar: bool = false;

/// Ratio of master area to stack area width
pub const mfact: f32 = 0.5;
/// Maximum number of clients in the master area
pub const nmaster: u32 = 2;

/// Layouts
pub const layouts: [Layout; 4] = [
    Layout { symbol: "[]=", arrange: tilearrange },
    Layout { symbol: "[M]", arrange: monoclearrange },
    Layout { symbol: "><>", arrange: noarrange },
    Layout { symbol: "HHH", arrange: gridarrange }
];

/// Tags
pub const tags: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Modifier key for key controls; Mod4Mask by default
pub const MODKEY: u32 = xlib::Mod4Mask;

/// Key combinations and their actions
pub const keys: [Key; 6] = [
    //    modifier              key                 function                argument
    Key { modif:MODKEY,                 keysym:XK_Return as u64, func:spawn, arg:Arg {s: "gnome-terminal -e tmux"}},
    Key { modif:MODKEY,                 keysym:XK_space as u64, func:spawn, arg:Arg {s: "rofi -show run"}},
    Key { modif:MODKEY|xlib::ShiftMask, keysym:XK_e as u64, func:quit, arg:Arg {i: 0}},
    Key { modif:MODKEY|xlib::ShiftMask, keysym:XF86XK_AudioLowerVolume as u64, func:spawn, arg:Arg {s: "amixer -q sset 'Master' 5%-"}},
    Key { modif:MODKEY|xlib::ShiftMask, keysym:XF86XK_AudioRaiseVolume as u64, func:spawn, arg:Arg {s: "amixer -q sset 'Master' 5%+"}},
    Key { modif:MODKEY|xlib::ShiftMask, keysym:XF86XK_AudioMute as u64, func:spawn, arg:Arg {s: "amixer -q sset 'Master' "}},
];

/// Buttons and their actions
pub const buttons: [Button; 0] = [

];
