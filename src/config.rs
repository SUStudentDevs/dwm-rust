use Layout;
use tilearrange;
use monoclearrange;
use noarrange;
use gridarrange;

// Appearance
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

// Layouts
pub const mfact: f32 = 0.5;     // Ratio of master area width
pub const nmaster: u32 = 2;     // Number of clients in master area

pub const layouts: [Layout; 4] = [
    Layout { symbol: "[]=", arrange: tilearrange },
    Layout { symbol: "[M]", arrange: monoclearrange },
    Layout { symbol: "><>", arrange: noarrange },
    Layout { symbol: "HHH", arrange: gridarrange }
];
