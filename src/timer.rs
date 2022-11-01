extern crate secfmt;

use gtk::glib;
use gtk::{Button, Label, SpinButton, Window};
use gtk::prelude::*;

pub fn recording_delay(delay_spin: SpinButton, mut delay_time: u64, delay_window: Window, delay_window_label: Label, record_button: Button) {
    // Keep time label alive and update every 1sec
    let default_value = delay_time;
    let capture_label = move || {
        // Show delay window if delay time is not zero
        delay_window.show();
        if delay_time > 0 {
            delay_window_label.set_text(&current_time(delay_time));
            delay_time -= 1;
            glib::source::Continue(true)
        } else {
            // Hide delay window and start recording
            delay_window.hide();
            delay_spin.set_value(0.0);
            record_button.emit_clicked();
            // Keep the input value
            delay_spin.set_value(default_value as f64);
            glib::source::Continue(false)
        }
    };
    // Execute capture_label every 1sec
    glib::source::timeout_add_seconds_local(1, capture_label);
}

fn current_time(delay_time: u64) -> String {
    let delay = secfmt::from(delay_time);
    format!("{}:{}", delay.minutes, delay.seconds)
}
