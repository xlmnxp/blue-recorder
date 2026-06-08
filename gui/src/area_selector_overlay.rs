// Lets the user pick a capture rectangle by dragging over a transparent,
// fullscreen overlay window — a GTK-native replacement for shelling out to
// `slurp` (which only works on wlroots-based Wayland compositors).

use crate::fluent::get_bundle;
use adw::gdk;
use adw::glib;
use adw::gtk::{self, prelude::*};
use std::cell::Cell;
use std::rc::Rc;

const OVERLAY_CSS: &[u8] = b"
window.area-selector-overlay {
    background-color: rgba(0, 0, 0, 0);
}
.area-selector-overlay-hint {
    padding: 6px 12px;
    border-radius: 6px;
    background-color: rgba(0, 0, 0, 0.6);
    color: #ffffff;
}
";

/// Selection box border/fill color, falling back to the selected-background
/// theme color's usual blue if the theme doesn't define it.
const FALLBACK_SELECTION_COLOR: (f32, f32, f32, f32) = (0.21, 0.52, 0.89, 1.0);

/// Shows a fullscreen transparent overlay on the monitor whose logical origin
/// is `(monitor_x, monitor_y)` and lets the user drag out a selection
/// rectangle. Returns `(x, y, width, height)` relative to that monitor's
/// logical origin, or `None` if the user cancelled (Escape) or the drag
/// produced an empty rectangle.
///
/// `logical_w`/`logical_h` are accepted to mirror the shape of the previous
/// `slurp`-based selector but aren't needed here — the overlay simply
/// fullscreens itself onto the target monitor.
pub fn select_area(
    monitor_x: i32,
    monitor_y: i32,
    _logical_w: i32,
    _logical_h: i32,
) -> Option<(u16, u16, u16, u16)> {
    let display = gdk::Display::default()?;
    let monitor = find_monitor(&display, monitor_x, monitor_y)?;

    let css = gtk::CssProvider::new();
    css.load_from_data(OVERLAY_CSS);
    gtk::StyleContext::add_provider_for_display(
        &display,
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = gtk::Window::new();
    window.set_decorated(false);
    window.add_css_class("area-selector-overlay");

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_cursor_from_name(Some("crosshair"));

    let hint_label = gtk::Label::new(Some(&get_bundle("area-selector-overlay-hint", None)));
    hint_label.add_css_class("area-selector-overlay-hint");
    hint_label.set_halign(gtk::Align::Center);
    hint_label.set_valign(gtk::Align::Center);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&drawing_area));
    overlay.add_overlay(&hint_label);
    window.set_child(Some(&overlay));

    // Resolve the selection box color from the theme so it matches the
    // desktop's accent color, falling back to a sensible default.
    let (sel_r, sel_g, sel_b, sel_a) = drawing_area
        .style_context()
        .lookup_color("theme_selected_bg_color")
        .map(|c| (c.red(), c.green(), c.blue(), c.alpha()))
        .unwrap_or(FALLBACK_SELECTION_COLOR);

    // (start_x, start_y, current_x, current_y) of the in-progress drag, in
    // window-local (== monitor-logical) coordinates.
    let drag_rect = Rc::new(Cell::new(None::<(f64, f64, f64, f64)>));
    let result = Rc::new(Cell::new(None::<(u16, u16, u16, u16)>));
    let main_loop = glib::MainLoop::new(None, false);

    drawing_area.set_draw_func({
        let drag_rect = drag_rect.clone();
        move |_area, cr, _width, _height| {
            let Some((sx, sy, cx, cy)) = drag_rect.get() else { return };
            let x = sx.min(cx);
            let y = sy.min(cy);
            let w = (cx - sx).abs();
            let h = (cy - sy).abs();

            cr.set_source_rgba(sel_r as f64, sel_g as f64, sel_b as f64, sel_a as f64 * 0.2);
            cr.rectangle(x, y, w, h);
            let _ = cr.fill_preserve();

            cr.set_source_rgba(sel_r as f64, sel_g as f64, sel_b as f64, sel_a as f64);
            cr.set_line_width(1.5);
            cr.rectangle(x, y, w, h);
            let _ = cr.stroke();
        }
    });

    let drag = gtk::GestureDrag::new();
    drawing_area.add_controller(&drag);

    drag.connect_drag_begin({
        let drag_rect = drag_rect.clone();
        let drawing_area = drawing_area.clone();
        let hint_label = hint_label.clone();
        move |_gesture, x, y| {
            hint_label.set_visible(false);
            drag_rect.set(Some((x, y, x, y)));
            drawing_area.queue_draw();
        }
    });
    drag.connect_drag_update({
        let drag_rect = drag_rect.clone();
        let drawing_area = drawing_area.clone();
        move |_gesture, offset_x, offset_y| {
            if let Some((sx, sy, _, _)) = drag_rect.get() {
                drag_rect.set(Some((sx, sy, sx + offset_x, sy + offset_y)));
                drawing_area.queue_draw();
            }
        }
    });
    drag.connect_drag_end({
        let drag_rect = drag_rect.clone();
        let result = result.clone();
        let main_loop = main_loop.clone();
        move |_gesture, offset_x, offset_y| {
            if let Some((sx, sy, _, _)) = drag_rect.get() {
                result.set(rect_from_drag(sx, sy, sx + offset_x, sy + offset_y));
            }
            main_loop.quit();
        }
    });

    let key_controller = gtk::EventControllerKey::new();
    window.add_controller(&key_controller);
    key_controller.connect_key_pressed({
        let result = result.clone();
        let main_loop = main_loop.clone();
        move |_controller, key, _keycode, _state| {
            if key == gdk::Key::Escape {
                result.set(None);
                main_loop.quit();
            }
            glib::signal::Inhibit(false)
        }
    });

    window.fullscreen_on_monitor(&monitor);
    window.present();

    main_loop.run();

    // Hide and tear the overlay down, then give the compositor real wall-clock
    // time to actually unmap and redraw without it before returning. Without
    // this, the screen-cast stream — already live by the time area selection
    // runs — can capture a frame that still shows our overlay, so it ends up
    // baked into the recording. Unmapping over Wayland is asynchronous, so
    // there's no synchronous "wait until gone"; a short timed pump is the
    // pragmatic fix (this matches the kind of delay slurp itself incurs by
    // being a separate process that has to exit and get unmapped first).
    window.set_visible(false);
    window.destroy();
    let settle_loop = glib::MainLoop::new(None, false);
    glib::source::timeout_add_local_once(std::time::Duration::from_millis(150), {
        let settle_loop = settle_loop.clone();
        move || settle_loop.quit()
    });
    settle_loop.run();

    result.take()
}

fn find_monitor(display: &gdk::Display, x: i32, y: i32) -> Option<gdk::Monitor> {
    let monitors = display.monitors();
    for i in 0..monitors.n_items() {
        let item = monitors.item(i)?;
        let monitor = item.downcast::<gdk::Monitor>().ok()?;
        let geometry = monitor.geometry();
        if geometry.x() == x && geometry.y() == y {
            return Some(monitor);
        }
    }
    None
}

/// Normalizes a drag's start/end points into a non-negative `(x, y, w, h)`
/// rectangle, or `None` if the drag never produced a non-empty area.
fn rect_from_drag(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Option<(u16, u16, u16, u16)> {
    let x = start_x.min(end_x).max(0.0).round() as u16;
    let y = start_y.min(end_y).max(0.0).round() as u16;
    let w = (end_x - start_x).abs().round() as u16;
    let h = (end_y - start_y).abs().round() as u16;
    if w == 0 || h == 0 {
        return None;
    }
    Some((x, y, w, h))
}
