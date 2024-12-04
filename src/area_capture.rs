extern crate regex;

use anyhow::{anyhow, Result};
use regex::Regex;
use std::process::Command;

// This struct use "xwininfo" in linux & freebsd to get area x, y, width and height
#[derive(Debug, Copy, Clone)]
pub struct AreaCapture {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[cfg(any(target_os = "freebsd", target_os = "linux"))]
impl AreaCapture {
    pub fn new() -> Result<AreaCapture> {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-root").output()?.stdout)?
        )?;

        let area_capture = AreaCapture {
            x: coordinate.0,
            y: coordinate.1,
            width: coordinate.2,
            height: coordinate.3,
        };
        Ok(area_capture)
    }

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

    pub fn get_window_by_name(&mut self, name: &str) -> Result<Self> {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-name").arg(name).output().unwrap().stdout).unwrap(),
        )?;
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        Ok(*self)
    }

    pub fn reset(&mut self) -> Result<Self> {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-root").output().unwrap().stdout).unwrap()
        )?;
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        Ok(*self)
    }
}

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
