# Blue Recorder [![Made With Rust](https://img.shields.io/static/v1?label=MADE%20WITH&message=Rust&color=red&style=for-the-badge&logo=Rust)](https://GitHub.com/xlmnxp/blue-recorder) [![GitHub contributors](https://img.shields.io/github/contributors/xlmnxp/blue-recorder.svg?style=for-the-badge)](https://GitHub.com/xlmnxp/blue-recorder/graphs/contributors)

<img src="screenshots/screenshot1.png"/>

A simple desktop recorder for Linux systems. Built using GTK4 and ffmpeg. 

- It supports recording audio and video on almost all Linux interfaces with support for Wayland display server on GNOME session.
- The following formats are currently supported: mkv, avi, mp4, wmv, gif and nut.
- You can stop the recording process easily by right-clicking the icon and choosing "Stop Record". Or middle-clicking the recording icon in the notifications area (but doesn't work on all interfaces).
- You can choose the audio input source you want from the list.
- You can also set the default values you want by simply changing them in the interface, and the program will save them for you for the next time you open it. 

This project is based on [Green Recorder](https://github.com/mhsabbagh/green-recorder) and is rewritten in Rust with improvements.

## Installation
Blue Recorder available as Snap and Flatpak package:

<a href='https://snapcraft.io/blue-recorder'><img height='60' alt='Get it from the Snap Store' src='https://snapcraft.io/static/images/badges/en/snap-store-black.svg'/></a> <a href='https://flathub.org/apps/details/sa.sy.bluerecorder'><img height='60' alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a>

## Build from source
Blue Recorder depend in the following dependencies:
```
ffmpeg
gtk
gdk
gio
x11-utils
```

Install dependencies Ubuntu and Debian based distros:
```
sudo apt install build-essential clang cargo x11-utils ffmpeg libgtk-3-dev libgtk-4-dev libatk1.0-dev libasound2-dev libgstreamer1.0-dev
```
Then use `Cargo` to build it:
```
git clone https://github.com/xlmnxp/blue-recorder
cd <Project Directory>
cargo build --release
cp -a data interfaces po target/release
```
Then you will find the executable file at: 
`<Project Directory>/target/release/blue-recorder`

## License
```
GNU General Public License version 3
License Copyright: Copyright © 2007 Free Software Foundation, Inc.
License License: "Everyone is permitted to copy and distribute verbatim copies of this license document, but changing it is not allowed." (www.gnu.org/licenses/gpl-3.0.en.html).
License Contact: Free Software Foundation (fsf.org).
SPDX short identifier: GPL-3.0-only
Further resources...
```
[Read the license](LICENSE.md)
