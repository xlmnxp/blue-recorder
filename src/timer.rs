extern crate secfmt;

use gtk::glib;
use gtk::{Button, ToggleButton, Label, SpinButton, Window};
use gtk::prelude::*;

pub fn recording_delay(delay_spin: SpinButton, mut delay_time: u64, delay_window: Window, delay_window_button: ToggleButton, delay_window_label: Label, record_button: Button) {
    // Keep time label alive and update every 1sec
    let default_value = delay_time;
    let capture_delay_label = move || {
        // Show delay window if delay time is not zero
        delay_window.show();
        if delay_time  > 0 {
            delay_window_label.set_text(&current_time(delay_time));
            delay_time -= 1;
            if delay_window_button.is_active() {
                delay_window.hide();
                glib::source::Continue(false)
            } else {
                glib::source::Continue(true)
            }
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
    glib::source::timeout_add_seconds_local(1, capture_delay_label);
}

pub fn start_timer(record_time_label: Label) {
    let mut start_time = 0;
    let capture_record_label = move || {
        if record_time_label.is_visible() {
            record_time_label.set_text(&current_time(start_time));
            start_time += 1;
            glib::source::Continue(true)
        } else {
            glib::source::Continue(false)
        }
    };
    // Execute capture_label every 1sec
    glib::source::timeout_add_seconds_local(1, capture_record_label);
}

pub fn stop_timer(record_time_label: Label) {
    let stop_time = 0;
    record_time_label.set_text(&current_time(stop_time));
}


fn current_time(delay_time: u64) -> String {
    let delay = secfmt::from(delay_time);
    format!("{:02}:{:02}", delay.minutes, delay.seconds)
}
