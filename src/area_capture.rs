extern crate regex;

use anyhow::{anyhow, Result};
use display_info::DisplayInfo;
use glib::Continue;
use libadwaita::gtk::Label;
use libadwaita::Window;
use libadwaita::prelude::*;
use regex::Regex;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
#[cfg(target_os = "windows")]
use x_win::get_active_window;

// This struct use "xwininfo" in linux & freebsd to get area x, y, width and height
#[derive(Debug, Copy, Clone)]
pub struct AreaCapture {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl AreaCapture {
    pub fn new() -> Result<AreaCapture> {
        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-root").output()?.stdout)?
        )?;

        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        let area_capture = AreaCapture {
            x: coordinate.0,
            y: coordinate.1,
            width: coordinate.2,
            height: coordinate.3,
        };

        #[cfg(target_os = "windows")]
        let coordinate = DisplayInfo::all()?;

        #[cfg(target_os = "windows")]
        let area_capture = AreaCapture {
            x: coordinate[0].x as u16,
            y: coordinate[0].y as u16,
            width: coordinate[0].width as u16,
            height: coordinate[0].height as u16,
        };
        Ok(area_capture)
    }

    #[cfg(target_os = "windows")]
    pub fn get_active_window(&mut self) -> Result<Self> {
        let coordinate = get_active_window()?.position;

        self.x = coordinate.x as u16;
        self.y = coordinate.y as u16;
        self.width = coordinate.width as u16;
        self.height = coordinate.height as u16;
        Ok(*self)
    }

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    pub fn get_area(&mut self) -> Result<Self> {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").output()?.stdout)?
        )?;
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        Ok(*self)
    }

    #[cfg(target_os = "windows")]
    pub fn get_title(&mut self) -> Result<String> {
        let title = get_active_window()?.title;
        Ok(title)
    }

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    pub fn get_window_by_name(&mut self, name: &str) -> Result<Self> {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-name").arg(name).output()?.stdout)?,
        )?;
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        Ok(*self)
    }

    pub fn reset(&mut self) -> Result<Self> {
        if cfg!(target_os = "windows") {
            let coordinate = DisplayInfo::all()?;
            self.x = coordinate[0].x as u16;
            self.y = coordinate[0].y as u16;
            self.width = coordinate[0].width as u16;
            self.height = coordinate[0].height as u16;
        } else {
            let coordinate = xwininfo_to_coordinate(
                String::from_utf8(Command::new("xwininfo").arg("-root").output()?.stdout)?
            )?;
            self.x = coordinate.0;
            self.y = coordinate.1;
            self.width = coordinate.2;
            self.height = coordinate.3;
        }

        Ok(*self)
    }
}

#[cfg(any(target_os = "freebsd", target_os = "linux"))]
fn xwininfo_to_coordinate(xwininfo_output: String) -> Result<(u16, u16, u16, u16)> {
    let x: u16 = Regex::new(r"A.*X:\s+(\d+)\n")?
        .captures(xwininfo_output.as_str())
        .ok_or_else(|| anyhow!("failed to capture string from xwininfo_output"))?
        .get(1)
        .ok_or_else(|| anyhow!("failed to get x value from xwininfo_output"))?
        .as_str()
        .to_string()
        .parse::<u16>()?;
    let y: u16 = Regex::new(r"A.*Y:\s+(\d+)\n")?
        .captures(xwininfo_output.as_str())
        .ok_or_else(|| anyhow!("failed to capture string from xwininfo_output"))?
        .get(1)
        .ok_or_else(|| anyhow!("failed to get y value from xwininfo_output"))?
        .as_str()
        .to_string()
        .parse::<u16>()?;
    let width: u16 = Regex::new(r"Width:\s(\d+)\n")?
        .captures(xwininfo_output.as_str())
        .ok_or_else(|| anyhow!("failed to capture string from xwininfo_output"))?
        .get(1)
        .ok_or_else(|| anyhow!("failed to get width value from xwininfo_output"))?
        .as_str()
        .to_string()
        .parse::<u16>()?;
    let height: u16 = Regex::new(r"Height:\s(\d+)\n")?
        .captures(xwininfo_output.as_str())
        .ok_or_else(|| anyhow!("failed to capture string from xwininfo_output"))?
        .get(1)
        .ok_or_else(|| anyhow!("failed to get height value from xwininfo_output"))?
        .as_str()
        .to_string()
        .parse::<u16>()?;

    Ok((x, y, width, height))
}

// Display area chooser window size
pub fn show_size(area_chooser_window: Window, area_size_bottom_label: Label, area_size_top_label: Label) -> Result<()> {
    // Create a shared state for the area size
    let size_labels = Rc::new(RefCell::new((area_size_top_label, area_size_bottom_label)));

    // Use a timeout to periodically check the window size
    glib::timeout_add_local(1000, {
        let area_chooser_window = area_chooser_window.clone();
        let size_labels = size_labels.clone();

        move || {
            if !area_chooser_window.is_active() {
                return Continue(false); // Stop the timeout
            }

            let mut area_capture = AreaCapture::new().unwrap();
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            let size = area_capture.get_window_by_name(
                area_chooser_window.title().unwrap().as_str()
            ).unwrap();

            #[cfg(target_os = "windows")]
            let size = area_capture.get_active_window().unwrap();

            // Update the labels
            let (top_label, bottom_label) = size_labels.borrow_mut().to_owned();
            top_label.set_text(&format!("{}x{}", size.width, size.height));
            bottom_label.set_text(&format!("{}x{}", size.width, size.height));

            Continue(true) // Continue the timeout
        }
    });

    Ok(())
}
