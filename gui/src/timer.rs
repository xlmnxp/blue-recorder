extern crate secfmt;

use adw::gtk::{Button, ToggleButton, Label, SpinButton};
use adw::gtk::glib;
use adw::gtk::prelude::*;
use adw::Window;
use std::cell::RefCell;
use std::rc::Rc;

// Check for record_button.emit_clicked()
#[derive(Clone, Copy)]
pub struct RecordClick {
    pub is_record_button_clicked: bool,
}

impl RecordClick {
    pub fn set_clicked(&mut self, status: bool) {
        self.is_record_button_clicked = status;
    }

    pub fn is_clicked(&mut self) -> bool {
        self.is_record_button_clicked
    }
}

pub fn recording_delay(delay_spin: SpinButton, mut delay_time: u16, delay_window: Window, delay_window_button: ToggleButton,
                       delay_window_label: Label, record_button: Button, record_click: Rc<RefCell<RecordClick>>) {
    // Keep time label alive and update every 1sec
    let default_value = delay_time;
    let capture_delay_label = move || {
        // Show delay window if delay time is not zero
        delay_window.show();
        if delay_time  > 0 {
            delay_window_label.set_text(&current_delay_time(delay_time));
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
            // Inform that record button is on second click
            record_click.borrow_mut().set_clicked(true);
            record_button.emit_clicked();
            // Revert record_click
            record_click.borrow_mut().set_clicked(false);
            // Keep the input value
            delay_spin.set_value(default_value as f64);
            glib::source::Continue(false)
        }
    };
    // Execute capture_label every 1sec
    glib::source::timeout_add_seconds_local(1, capture_delay_label);
}

pub fn start_timer(record_time_label: Label) {
    let mut start_time = 1;
    let capture_record_label = move || {
        if record_time_label.is_visible() {
            record_time_label.set_text(&current_record_time(start_time));
            start_time += 1;
            glib::source::Continue(true)
        } else {
            glib::source::Continue(false)
        }
    };
    // Execute capture_record_label every 1sec
    glib::source::timeout_add_seconds_local(1, capture_record_label);
}

pub fn stop_timer(record_time_label: Label) {
    let stop_time = 0;
    record_time_label.set_text(&current_record_time(stop_time));
}

fn current_delay_time(delay_time: u16) -> String {
    let delay = secfmt::from(delay_time as u64);
    format!("{:02}", delay.seconds)
}

fn current_record_time(start_time: u16) -> String {
    let start = secfmt::from(start_time as u64);
    format!("{:02}:{:02}:{:02}", start.hours, start.minutes, start.seconds)
}
