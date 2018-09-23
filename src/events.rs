
use std::ptr;

use x11::xlib;

use wm;
use wm::WM;
use wm::client;

use config;

/**
 * Handles an event
 */
pub fn handleEvent<'a>(wm: WM<'a>, ev: &xlib::XEvent) -> WM<'a> {
    unsafe {
        match ev.type_ {
            xlib::ConfigureRequest => configureRequest(wm, ev),
            xlib::ConfigureNotify => configureNotify(wm, ev),
            //xlib::EnterNotify => enternotify(wm, ev),
            xlib::DestroyNotify => destroyNotify(wm, ev),
            xlib::KeyPress => keyPress(wm, ev),
            xlib::ButtonPress => buttonPress(wm, ev),
            xlib::MapRequest => mapRequest(wm, ev),
            xlib::PropertyNotify => propertyNotify(wm, ev),
            // TODO : les autres handlers
            _ => wm
        }
    }
}

/**
 * Handles a ConfigureRequest event : before changing the configuration of a window
 */
pub fn configureRequest<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.configure_request };
    if let Some(c) = client::findFromWindow(ev.window, &wm.wss) {
        client::configure(c, wm.drw.dpy);
    } else {
        let mut wc = xlib::XWindowChanges {
            x: ev.x, y: ev.y,
            width: ev.width, height: ev.height,
            border_width: ev.border_width,
            sibling: ev.above,
            stack_mode: ev.detail
        };
        unsafe { xlib::XConfigureWindow(wm.drw.dpy, ev.window, ev.value_mask as u32, &mut wc) };
    }
    unsafe { xlib::XSync(wm.drw.dpy, 0) };
    wm
}

/**
 * Handles a ConfigureNotify event : after reconfiguration of a window
 */
pub fn configureNotify<'a>(wm : WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let _ev = unsafe { e.configure };
    // TODO
    wm::updateStatus(wm)
}

// /**
//  * Handles an EnterNotify event
//  */
// pub fn enternotify(wm: &mut WM, e: &xlib::XEvent) {
//     let ev = unsafe { e.crossing };
//     if (ev.mode != xlib::NotifyNormal || ev.detail == xlib::NotifyInferior) && ev.window != wm.root {
//         return;
//     }
//     let mut c = Client::from(ev.window, &wm.mons);
//     let m = if let Some(ref mut cl) = c {
//         &wm.mons[cl.monindex]
//     } else {
//         Workspace::from_window(ev.window, wm.root, &wm.mons, &wm.mons[wm.selmonindex])
//     };
//     if m != &wm.mons[wm.selmonindex] {
//         // unfocus(selmon.sel, true); TODO
//         wm.selmonindex = m.num as usize;
//     } else {
//         let mut c = Client::from(ev.window, &wm.mons);
//         let selmon = &wm.mons[wm.selmonindex];
//         match (c, selmon.sel) {
//             (None, _) => return,
//             (Some(cl), Some(sel)) => if cl == sel { return },
//             _ => ()
//         }
//     }
//     if let Some(cl) = c {
//         // focus(cl); TODO
//     }
// }

/**
 * Handles Window destruction
 */
pub fn destroyNotify<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.destroy_window };
    wm::updateStatus(wm::unManage(wm, ev.window))
}

fn cleanmask(mask: u32) -> u32 {
    mask
}

/**
 * Handles a KeyPress event
 */
pub fn keyPress<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.key };
    let keysym = unsafe { xlib::XKeycodeToKeysym(wm.drw.dpy, ev.keycode as u8, 0) };
    for i in 0..config::keys.len() {
        if keysym == config::keys[i].keysym
        && cleanmask(ev.state) == cleanmask(config::keys[i].modif) {
            let func = config::keys[i].func;
            return func(&config::keys[i].arg, wm);
        }
    }
    wm
}

/**
 * Handles a button press
 */
pub fn buttonPress<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let _ev = unsafe { e.button };
    // TODO
    wm
}

/**
 * Handles a MapRequest event
 */
pub fn mapRequest<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.map_request };
    let mut wa = xlib::XWindowAttributes { // Dummy value
        x: 0, y: 0, width: 0, height: 0, border_width: 0, depth: 0, visual: ptr::null_mut(), root: wm.root, class: 0, bit_gravity: 0, win_gravity: 0, backing_store: 0, backing_planes: 0, backing_pixel: 0, save_under: 0, colormap: 0, map_installed: 0, map_state: 0, all_event_masks: 0, your_event_mask: 0, do_not_propagate_mask: 0, override_redirect: 0, screen: ptr::null_mut()
    };
    if unsafe { xlib::XGetWindowAttributes(wm.drw.dpy, ev.window, &mut wa) } == 0 || wa.override_redirect != 0 {
        wm
    } else if client::findFromWindow(ev.window, &wm.wss) == None {
        return wm::manage(wm, ev.window, wa);
    } else {
        wm
    }
}

/**
 * Handles a Property Notify event
 */
pub fn propertyNotify<'a>(wm: WM<'a>, e: &xlib::XEvent) -> WM<'a> {
    let ev = unsafe { e.property };
    if ev.window == wm.root { wm::updateStatus(wm) } else { wm }
}
