extern crate gdk;
extern crate gettextrs;
extern crate gio;
extern crate gtk;
extern crate libappindicator;
mod area_capture;
mod config_management;
mod ffmpeg_interface;

// use gio::prelude::*;
use gettextrs::{bindtextdomain, gettext, setlocale, textdomain, LocaleCategory};
use glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::ComboBoxText;
use gtk::{
    AboutDialog, Builder, Button, CheckButton, CssProvider, Entry, FileChooser, Label, MenuItem,
    SpinButton, Window,
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::cell::RefCell;
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;

fn main() {
    // use "GDK_BACKEND=x11" to make xwininfo work in Wayland by using XWayland
    std::env::set_var("GDK_BACKEND", "x11");
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let builder: Builder;
    let user_interface_path_abs = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("interfaces/main.ui"));

    if user_interface_path_abs.exists() {
        builder = Builder::from_file(user_interface_path_abs);
    } else {
        builder = Builder::from_file("interfaces/main.ui");
    }

    // translate
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(
        "blue-recorder",
        std::fs::canonicalize(Path::new("po"))
            .unwrap()
            .to_str()
            .unwrap(),
    );
    textdomain("blue-recorder");

    // config initialize
    config_management::initialize();

    // get Objects from UI
    let main_window: Window = builder.get_object("main_window").unwrap();
    let about_dialog: AboutDialog = builder.get_object("about_dialog").unwrap();
    let area_chooser_window: Window = builder.get_object("area_chooser_window").unwrap();
    let folder_chooser: FileChooser = builder.get_object("filechooser").unwrap();
    let filename_entry: Entry = builder.get_object("filename").unwrap();
    let command_entry: Entry = builder.get_object("command").unwrap();
    let format_chooser_combobox: ComboBoxText = builder.get_object("comboboxtext1").unwrap();
    let audio_source_combobox: ComboBoxText = builder.get_object("audiosource").unwrap();
    let record_button: Button = builder.get_object("recordbutton").unwrap();
    let stop_button: Button = builder.get_object("stopbutton").unwrap();
    let play_button: Button = builder.get_object("playbutton").unwrap();
    let window_grab_button: Button = builder.get_object("window_grab_button").unwrap();
    let area_grab_button: Button = builder.get_object("area_grab_button").unwrap();
    let area_set_button: Button = builder.get_object("area_set_button").unwrap();
    let frames_label: Label = builder.get_object("frames_label").unwrap();
    let delay_label: Label = builder.get_object("delay_label").unwrap();
    let command_label: Label = builder.get_object("command_label").unwrap();
    let frames_spin: SpinButton = builder.get_object("frames").unwrap();
    let delay_spin: SpinButton = builder.get_object("delay").unwrap();
    let audio_source_label: Label = builder.get_object("audio_source_label").unwrap();
    let video_switch: CheckButton = builder.get_object("videoswitch").unwrap();
    let audio_switch: CheckButton = builder.get_object("audioswitch").unwrap();
    let mouse_switch: CheckButton = builder.get_object("mouseswitch").unwrap();
    let follow_mouse_switch: CheckButton = builder.get_object("followmouseswitch").unwrap();
    let about_menu_item: MenuItem = builder.get_object("about_menu_item").unwrap();

    // --- default properties
    // Windows
    main_window.set_title(&gettext("Blue Recorder"));
    // TODO: make area chooser window transparent
    // NOTICE: it work as snap package
    area_chooser_window.set_title(&gettext("Area Chooser"));
    area_chooser_window.set_visual(Some(
        &gdk::Screen::get_rgba_visual(&gdk::Screen::get_default().unwrap()).unwrap(),
    ));

    // Entries
    filename_entry.set_placeholder_text(Some(&gettext("Default filename:")));
    command_entry.set_placeholder_text(Some(&gettext("Default command:")));
    filename_entry.set_text(&config_management::get("default", "filename"));
    command_entry.set_text(&config_management::get("default", "command"));

    // CheckBox
    format_chooser_combobox.append(
        Some("mkv"),
        &gettext("MKV (Matroska multimedia container format)"),
    );
    format_chooser_combobox.append(Some("avi"), &gettext("AVI (Audio Video Interleaved)"));
    format_chooser_combobox.append(Some("mp4"), &gettext("MP4 (MPEG-4 Part 14)"));
    format_chooser_combobox.append(Some("wmv"), &gettext("WMV (Windows Media Video)"));
    // TODO: gif not work at this time, fix it!
    // format_chooser_combobox.append(Some("gif"), &gettext("GIF (Graphics Interchange Format)"));
    format_chooser_combobox.append(Some("nut"), &gettext("NUT (NUT Recording Format)"));
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

    audio_source_combobox.append(Some("default"), &gettext("Default PulseAudio Input Source"));
    for (id, audio_source) in sources_descriptions.iter().enumerate() {
        audio_source_combobox.append(Some(id.to_string().as_str()), audio_source);
    }
    audio_source_combobox.set_active(Some(0));

    // Switchs
    video_switch.set_label(&gettext("Record Video"));
    audio_switch.set_label(&gettext("Record Audio"));
    mouse_switch.set_label(&gettext("Show Mouse"));
    follow_mouse_switch.set_label(&gettext("Follow Mouse"));
    video_switch.set_active(config_management::get_bool("default", "videocheck"));
    audio_switch.set_active(config_management::get_bool("default", "audiocheck"));
    mouse_switch.set_active(config_management::get_bool("default", "mousecheck"));
    follow_mouse_switch.set_active(config_management::get_bool("default", "followmousecheck"));

    let _mouse_switch = mouse_switch.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    video_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "videocheck", switch.get_active());
        if switch.get_active() {
            _mouse_switch.set_sensitive(true);
            _follow_mouse_switch.set_sensitive(true);
        } else {
            _mouse_switch.set_sensitive(false);
            _follow_mouse_switch.set_sensitive(false);
        }
    });
    let _follow_mouse_switch = follow_mouse_switch.clone();
    mouse_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "mousecheck", switch.get_active());
        if switch.get_active() {
            _follow_mouse_switch.set_sensitive(true);
        } else {
            _follow_mouse_switch.set_sensitive(false);
        }
    });
    audio_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "audiocheck", switch.get_active());
    });
    follow_mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "followmousecheck", switch.get_active());
    });

    // About Dialog
    about_menu_item.set_label("about");
    about_dialog.set_transient_for(Some(&main_window));
    about_dialog.set_program_name(&gettext("Blue Recorder"));
    about_dialog.set_version(Some("3.2.3"));
    about_dialog.set_copyright(Some("© 2021 Salem Yaslem"));
    about_dialog.set_wrap_license(true);
    about_dialog.set_license(Some("Blue Recorder is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\nBlue Recorder is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.\n\nSee the GNU General Public License for more details. You should have received a copy of the GNU General Public License along with Blue Recorder. If not, see <http://www.gnu.org/licenses/>."));
    about_dialog.set_comments(Some(&gettext(
        "A simple screen recorder for Linux desktop. Supports Wayland & Xorg.",
    )));
    about_dialog.set_authors(&[
        "Salem Yaslem <s@sy.sa>",
        "M.Hanny Sabbagh <mhsabbagh@outlook.com>",
        "Alessandro Toia <gort818@gmail.com>",
        "Patreon Supporters: Ahmad Gharib, Medium,\nWilliam Grunow, Alex Benishek.",
    ]);
    about_dialog.set_artists(&["Mustapha Assabar"]);
    about_dialog.set_website(Some("https://github.com/xlmnxp/blue-recorder/"));
    about_dialog.set_logo_icon_name(Some("blue-recorder"));
    about_dialog.set_transient_for(Some(&main_window));

    // Buttons
    window_grab_button.set_label(&gettext("Select a Window"));
    area_grab_button.set_label(&gettext("Select an Area"));

    // Labels
    command_label.set_label(&gettext("Run Command After Recording:"));
    frames_label.set_label(&gettext("Frames:"));
    delay_label.set_label(&gettext("Delay:"));
    audio_source_label.set_label(&gettext("Audio Input Source:"));

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
    let _frames_spin = frames_spin.to_owned();
    frames_spin.connect_value_changed(move |_| {
        config_management::set(
            "default",
            "frame",
            _frames_spin.get_value().to_string().as_str(),
        );
    });
    let _delay_spin = delay_spin.to_owned();
    delay_spin.connect_value_changed(move |_| {
        config_management::set(
            "default",
            "delay",
            _delay_spin.get_value().to_string().as_str(),
        );
    });

    // Other
    folder_chooser.set_uri(&config_management::get("default", "folder"));

    // --- connections
    // show dialog window when about button clicked then hide it after close
    let _about_dialog: AboutDialog = about_dialog.to_owned();
    about_menu_item.connect_activate(move |_| {
        _about_dialog.run();
        _about_dialog.hide();
    });

    // Buttons
    let area_capture: Rc<RefCell<area_capture::AreaCapture>> =
        Rc::new(RefCell::new(area_capture::AreaCapture::new()));
    let mut _area_capture = area_capture.clone();
    window_grab_button.connect_clicked(move |_| {
        _area_capture.borrow_mut().get_area();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    area_grab_button.connect_clicked(move |_| {
        _area_chooser_window.show();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    area_set_button.connect_clicked(move |_| {
        _area_capture
            .borrow_mut()
            .get_window_by_name(&gettext("Area Chooser"));
        _area_chooser_window.hide();
    });

    // init record struct
    let ffmpeg_record_interface: Rc<RefCell<ffmpeg_interface::Ffmpeg>> =
        Rc::new(RefCell::new(ffmpeg_interface::Ffmpeg {
            filename: (folder_chooser, filename_entry, format_chooser_combobox),
            record_video: video_switch,
            record_audio: audio_switch,
            audio_id: audio_source_combobox,
            record_mouse: mouse_switch,
            follow_mouse: follow_mouse_switch,
            record_frames: frames_spin,
            record_delay: delay_spin,
            command: command_entry,
            process_id: None,
            saved_filename: None,
            unbound: None,
        }));

    // App Indicator
    let mut indicator_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/blue-recorder.png"));

    if !indicator_icon_path.exists() {
        indicator_icon_path = std::fs::canonicalize(Path::new("data/blue-recorder.png")).unwrap();
    }

    let indicator = Rc::new(RefCell::new(AppIndicator::new(
        "Blue Recorder",
        indicator_icon_path.to_str().unwrap(),
    )));
    indicator
        .clone()
        .borrow_mut()
        .set_status(AppIndicatorStatus::Passive);
    let mut menu = gtk::Menu::new();
    let indicator_stop_recording = gtk::MenuItem::with_label(&gettext("stop recording"));
    menu.append(&indicator_stop_recording);
    menu.show_all();
    indicator.clone().borrow_mut().set_menu(&mut menu);
    // when indictor stop recording button clicked
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let mut _indicator = indicator.clone();
    indicator_stop_recording.connect_activate(move |_| {
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        _indicator
            .borrow_mut()
            .set_status(AppIndicatorStatus::Passive);
    });

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let mut _area_capture = area_capture.clone();
    let mut _indicator = indicator.clone();
    record_button.connect_clicked(move |_| {
        let _area_capture = _area_capture.borrow_mut().clone();
        _ffmpeg_record_interface.borrow_mut().start_record(
            _area_capture.x,
            _area_capture.y,
            _area_capture.width,
            _area_capture.height,
        );
        _indicator
            .borrow_mut()
            .set_status(AppIndicatorStatus::Active);
    });

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let mut _indicator = indicator.clone();
    stop_button.connect_clicked(move |_| {
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        _indicator
            .borrow_mut()
            .set_status(AppIndicatorStatus::Passive);
    });

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    play_button.connect_clicked(move |_| {
        _ffmpeg_record_interface.borrow_mut().clone().play_record();
    });

    // Windows
    // hide area chooser after it deleted.
    let _area_chooser_window = area_chooser_window.clone();
    area_chooser_window.connect_delete_event(move |_, _event: &gdk::Event| {
        _area_chooser_window.hide();
        Inhibit(true)
    });

    // close the application when main window destroy
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let mut _indicator = indicator.clone();
    main_window.connect_destroy(move |_| {
        // stop recording before close the application
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        _indicator
            .borrow_mut()
            .set_status(AppIndicatorStatus::Passive);
        gtk::main_quit();
    });

    // apply css
    let provider = CssProvider::new();
    provider
        .load_from_data(include_str!("styles/global.css").as_bytes())
        .unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    gtk::main();
}
