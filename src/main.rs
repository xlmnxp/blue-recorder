extern crate gio;
extern crate gtk;
mod config_management;
mod signal_handle;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::ComboBoxText;
use gtk::{
    AboutDialog, Adjustment, Builder, Button, CheckButton, Entry, FileChooser, Label, MenuItem,
    SpinButton, Window,
};
use std::path::Path;
use std::process::{Command, Stdio};
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let builder: Builder = Builder::from_file(Path::new("windows/ui.glade"));

    config_management::initialize();
    // get Objects from UI
    let main_window: Window = builder.get_object("main_window").unwrap();
    let about_dialog: AboutDialog = builder.get_object("about_dialog").unwrap();
    let area_chooser: Window = builder.get_object("area_chooser_window").unwrap();
    let folder_chooser: FileChooser = builder.get_object("filechooser").unwrap();
    let filename_entry: Entry = builder.get_object("filename").unwrap();
    let command_entry: Entry = builder.get_object("command").unwrap();
    let format_chooser_combobox: ComboBoxText = builder.get_object("comboboxtext1").unwrap();
    let audio_source_combobox: ComboBoxText = builder.get_object("audiosource").unwrap();
    let record_button: Button = builder.get_object("recordbutton").unwrap();
    let stop_button: Button = builder.get_object("stopbutton").unwrap();
    let window_grab_button: Button = builder.get_object("window_grab_button").unwrap();
    let area_grab_button: Button = builder.get_object("area_grab_button").unwrap();
    let frames_label: Label = builder.get_object("frames_label").unwrap();
    let delay_label: Label = builder.get_object("delay_label").unwrap();
    let command_label: Label = builder.get_object("command_label").unwrap();
    let frames_spin: SpinButton = builder.get_object("frames").unwrap();
    let delay_spin: SpinButton = builder.get_object("delay").unwrap();
    let audio_source_label: Label = builder.get_object("audio_source_label").unwrap();
    let delay_adjustment: Adjustment = builder.get_object("adjustment1").unwrap();
    let frames_adjustment: Adjustment = builder.get_object("adjustment2").unwrap();
    let delay_pref_adjustment: Adjustment = builder.get_object("adjustment3").unwrap();
    let play_button: Button = builder.get_object("playbutton").unwrap();
    let video_switch: CheckButton = builder.get_object("videoswitch").unwrap();
    let audio_switch: CheckButton = builder.get_object("audioswitch").unwrap();
    let mouse_switch: CheckButton = builder.get_object("mouseswitch").unwrap();
    let follow_mouse_switch: CheckButton = builder.get_object("followmouseswitch").unwrap();
    let about_menu_item: MenuItem = builder.get_object("about_menu_item").unwrap();

    // --- default properties
    // Windows
    main_window.set_title("Blue Recorder");
    // Entries
    filename_entry.set_placeholder_text(Some("Enter filename"));
    command_entry.set_placeholder_text(Some("Enter your command here"));
    filename_entry.set_text(&config_management::get("default", "filename"));
    command_entry.set_text(&config_management::get("default", "command"));

    // CheckBox
    format_chooser_combobox.append(Some("mkv"), "MKV (Matroska multimedia container format)");
    format_chooser_combobox.append(Some("avi"), "AVI (Audio Video Interleaved)");
    format_chooser_combobox.append(Some("mp4"), "MP4 (MPEG-4 Part 14)");
    format_chooser_combobox.append(Some("wmv"), "WMV (Windows Media Video)");
    format_chooser_combobox.append(Some("gif"), "GIF (Graphics Interchange Format)");
    format_chooser_combobox.append(Some("nut"), "NUT (NUT Recording Format)");
    format_chooser_combobox.set_active(Some(0));
    // get audio sources
    let sources_descriptions: Vec<String> = {
        let sources_descriptions = String::from_utf8(
            Command::new("grep")
                .args(&["-e", "device.description"])
                .stdin(
                    Command::new("pactl")
                        .args(&["list", "sources"])
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap()
                        .stdout
                        .take()
                        .unwrap(),
                )
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        sources_descriptions
            .split("\n")
            .map(|s| {
                s.trim()
                    .replace("device.description = ", "")
                    .replace("\"", "")
            })
            .filter(|s| s != "")
            .collect()
    };

    for (id, audio_source) in sources_descriptions.iter().enumerate() {
        audio_source_combobox.append(Some(id.to_string().as_str()), audio_source);
    }
    audio_source_combobox.set_active(Some(0));

    // Switchs
    video_switch.set_label("Record Video");
    audio_switch.set_label("Record Audio");
    mouse_switch.set_label("Show Mouse");
    follow_mouse_switch.set_label("Follow Mouse");
    video_switch.set_active(config_management::get_bool("default", "videocheck"));
    audio_switch.set_active(config_management::get_bool("default", "audiocheck"));
    mouse_switch.set_active(config_management::get_bool("default", "mousecheck"));
    follow_mouse_switch.set_active(config_management::get_bool("default", "followmousecheck"));
    video_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "videocheck", switch.get_active());
    });
    audio_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "audiocheck", switch.get_active());
    });
    mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "mousecheck", switch.get_active());
    });
    follow_mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "followmousecheck", switch.get_active());
    });

    // About Dialog
    about_menu_item.set_label("about");
    about_dialog.set_transient_for(Some(&main_window));
    about_dialog.set_program_name("Blue Recorder");
    about_dialog.set_version(Some("3.2.3"));
    about_dialog.set_copyright(Some("© 2021 Salem Yaslem"));
    about_dialog.set_wrap_license(true);
    about_dialog.set_license(Some("Blue Recorder is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\nBlue Recorder is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.\n\nSee the GNU General Public License for more details. You should have received a copy of the GNU General Public License along with Blue Recorder. If not, see <http://www.gnu.org/licenses/>."));
    about_dialog.set_comments(Some(
        "A simple screen recorder for Linux desktop. Supports Wayland & Xorg.",
    ));
    about_dialog.set_authors(&[
        "Salem Yaslem <s@sy.sa>",
        "M.Hanny Sabbagh <mhsabbagh@outlook.com>",
        "Alessandro Toia <gort818@gmail.com>",
        "Patreon Supporters: Ahmad Gharib, Medium,\nWilliam Grunow, Alex Benishek.",
    ]);
    about_dialog.set_artists(&["Mustapha Assabar"]);
    about_dialog.set_website(Some("https://github.com/xlmnxp/blue-recorder/"));
    about_dialog.set_logo_icon_name(Some("blue-recorder"));

    // Buttons
    window_grab_button.set_label("Select a Window");
    area_grab_button.set_label("Select an Area");

    // Labels
    command_label.set_label("Run Command After Recording");
    frames_label.set_label("Frames");
    delay_label.set_label("Delay");
    audio_source_label.set_label("Audio Input Source");

    // Spin
    frames_spin.set_value(
        config_management::get("default", "frame")
            .to_string()
            .parse::<f64>()
            .unwrap(),
    );
    delay_spin.set_value(
        config_management::get("default", "delay")
            .to_string()
            .parse::<f64>()
            .unwrap(),
    );

    // Other
    folder_chooser.set_uri(&config_management::get("default", "folder"));

    // --- connections
    // show dialog window when about button clicked then hide it after close
    {
        let about_dialog: AboutDialog = about_dialog.to_owned();
        about_menu_item.connect_activate(move |_| {
            about_dialog.run();
            about_dialog.hide();
        });
    }

    // close the application when main window destroy
    main_window.connect_destroy(|_| {
        std::process::exit(0);
    });
    gtk::main();
}
