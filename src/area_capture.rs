extern crate regex;
use regex::Regex;
use std::process::Command;

// This struct use "xwininfo" to get area x, y, width and height
#[derive(Debug, Copy, Clone)]
pub struct AreaCapture {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl AreaCapture {
    pub fn new() -> AreaCapture {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-root").output().unwrap().stdout).unwrap()
        );

        AreaCapture {
            x: coordinate.0,
            y: coordinate.1,
            width: coordinate.2,
            height: coordinate.3,
        }
    }

    pub fn get_area(&mut self) -> Self {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").output().unwrap().stdout).unwrap()
        );
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        *self
    }

    pub fn get_window_by_name(&mut self, name: &str) -> Self {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-name").arg(name).output().unwrap().stdout).unwrap(),
        );
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        *self
    }

    pub fn reset(&mut self) -> Self {
        let coordinate = xwininfo_to_coordinate(
            String::from_utf8(Command::new("xwininfo").arg("-root").output().unwrap().stdout).unwrap()
        );
        self.x = coordinate.0;
        self.y = coordinate.1;
        self.width = coordinate.2;
        self.height = coordinate.3;
        *self
    }
}

fn xwininfo_to_coordinate(xwininfo_output: String) -> (u16, u16, u16, u16) {
    let x: u16 = Regex::new(r"A.*X:\s+(\d+)\n")
        .unwrap()
        .captures(xwininfo_output.as_str())
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
        .parse::<u16>()
        .unwrap();
    let y: u16 = Regex::new(r"A.*Y:\s+(\d+)\n")
        .unwrap()
        .captures(xwininfo_output.as_str())
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
        .parse::<u16>()
        .unwrap();
    let width: u16 = Regex::new(r"Width:\s(\d+)\n")
        .unwrap()
        .captures(xwininfo_output.as_str())
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
        .parse::<u16>()
        .unwrap();
    let height: u16 = Regex::new(r"Height:\s(\d+)\n")
        .unwrap()
        .captures(xwininfo_output.as_str())
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
        .parse::<u16>()
        .unwrap();

    (x, y, width, height)
}
