extern crate gio;
extern crate gdk;
extern crate gettextrs;
extern crate gtk;
mod area_capture;
mod config_management;
mod ffmpeg_interface;

use ffmpeg_interface::{Ffmpeg, ProgressWidget};
use gettextrs::{bindtextdomain, gettext, LocaleCategory, setlocale, textdomain};
use gtk::{prelude::*, Application};
use gtk::{AboutDialog, Builder, Button, CheckButton, ComboBoxText, CssProvider, Entry, FileChooserNative, FileChooserAction, Image, Label, MessageDialog, ProgressBar, SpinButton, ToggleButton, Window};
use std::cell::RefCell;
use std::ops::Add;
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;

fn main() {
    //Create new application
    let application = Application::new(Some("sa.sy.blue-recorder"), Default::default(),);
    application.connect_activate(build_ui);
    application.run();
}

pub fn build_ui(application: &Application) {
    // Use "GDK_BACKEND=x11" to make xwininfo work in Wayland by using XWayland
    std::env::set_var("GDK_BACKEND", "x11");
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let ui_src = include_str!("../interfaces/main.ui").to_string();
    let builder: Builder = Builder::from_string(ui_src.as_str());

   // Translate
    let mut po_path_abs = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("po"));

    if !po_path_abs.exists() {
        po_path_abs = std::fs::canonicalize(Path::new(
            &std::env::var("PO_DIR").unwrap_or_else(|_| String::from("po")),
        ))
        .unwrap();
    }

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("blue-recorder", po_path_abs.to_str().unwrap()).unwrap();
    textdomain("blue-recorder").unwrap();

    // Config initialize
    config_management::initialize();

    // Get Objects from UI
    let area_chooser_window: Window = builder.object("area_chooser_window").unwrap();
    let area_grab_button: ToggleButton = builder.object("area_grab_button").unwrap();
    let area_grab_icon: Image = builder.object("area_grab_icon").unwrap();
    let area_set_button: Button = builder.object("area_set_button").unwrap();
    let about_button: Button = builder.object("aboutbutton").unwrap();
    let about_dialog: AboutDialog = builder.object("about_dialog").unwrap();
    let audio_source_combobox: ComboBoxText = builder.object("audiosource").unwrap();
    let audio_source_label: Label = builder.object("audio_source_label").unwrap();
    let audio_switch: CheckButton = builder.object("audioswitch").unwrap();
    let command_entry: Entry = builder.object("command").unwrap();
    let command_label: Label = builder.object("command_label").unwrap();
    let delay_label: Label = builder.object("delay_label").unwrap();
    let delay_spin: SpinButton = builder.object("delay").unwrap();
    let filename_entry: Entry = builder.object("filename").unwrap();
    let folder_chooser_button: Button = builder.object("folder_chooser").unwrap();
    let folder_chooser_image: Image = builder.object("folder_chooser_image").unwrap();
    let folder_chooser_label: Label = builder.object("folder_chooser_label").unwrap();
    let follow_mouse_switch: CheckButton = builder.object("followmouseswitch").unwrap();
    let format_chooser_combobox: ComboBoxText = builder.object("comboboxtext1").unwrap();
    let frames_label: Label = builder.object("frames_label").unwrap();
    let frames_spin: SpinButton = builder.object("frames").unwrap();
    let main_window: Window = builder.object("main_window").unwrap();
    let mouse_switch: CheckButton = builder.object("mouseswitch").unwrap();
    let overwrite_switch: CheckButton = builder.object("overwriteswitch").unwrap();
    let play_button: Button = builder.object("playbutton").unwrap();
    let progress_dialog: MessageDialog = builder.object("progress_dialog").unwrap();
    let progressbar: ProgressBar = builder.object("progressbar").unwrap();
    let record_button: Button = builder.object("recordbutton").unwrap();
    let screen_grab_button: ToggleButton = builder.object("screen_grab_button").unwrap();
    let screen_grab_icon: Image = builder.object("screen_grab_icon").unwrap();
    let stop_button: Button = builder.object("stopbutton").unwrap();
    let video_switch: CheckButton = builder.object("videoswitch").unwrap();
    let window_grab_icon: Image = builder.object("window_grab_icon").unwrap();
    let window_grab_button: ToggleButton = builder.object("window_grab_button").unwrap();

    // --- default properties
    // Windows
    main_window.set_title(Some(&gettext("Blue Recorder")));
    main_window.set_application(Some(application));
    area_chooser_window.set_title(Some(&gettext("Area Chooser"))); //title is hidden

    //Hide stop & play buttons
    stop_button.hide();
    play_button.hide();

    //Hide window grab button in Wayland
    if is_wayland() {
        window_grab_button.hide();
    }

    // Entries
    filename_entry.set_placeholder_text(Some(&gettext("Default filename:")));
    command_entry.set_placeholder_text(Some(&gettext("Default command:")));
    filename_entry.set_text(&config_management::get("default", "filename"));
    command_entry.set_text(&config_management::get("default", "command"));

    // CheckBox
    //format_chooser_combobox.append(Some("webm"), &gettext("WEBM (Open Web Media File)"));
    format_chooser_combobox.append(Some("mp4"), &gettext("MP4 (MPEG-4 Part 14)"));
    //format_chooser_combobox.append(Some("gif"), &gettext("GIF (Graphics Interchange Format)"));
    format_chooser_combobox.append(
        Some("mkv"),
        &gettext("MKV (Matroska multimedia container format)"),
    );
    //format_chooser_combobox.append(Some("avi"), &gettext("AVI (Audio Video Interleaved)"));
    //format_chooser_combobox.append(Some("wmv"), &gettext("WMV (Windows Media Video)"));
    //format_chooser_combobox.append(Some("nut"), &gettext("NUT (NUT Recording Format)"));
    format_chooser_combobox.set_active(Some(0));

    // Get audio sources
    let sources_descriptions: Vec<String> = {
        let list_sources_child = Command::new("pactl")
        .args(&["list", "sources"])
        .stdout(Stdio::piped())
        .spawn();
        let sources_descriptions = String::from_utf8(
            if let Ok(..) = list_sources_child {
            Command::new("grep")
                .args(&["-e", "device.description"])
                .stdin(
                    list_sources_child
                    .unwrap()
                    .stdout
                    .take()
                    .unwrap(),
                )
                .output()
                .unwrap()
                .stdout
            } else {
                Vec::new()
            }
        )
        .unwrap();
        sources_descriptions
            .split('\n')
            .map(|s| {
                s.trim()
                    .replace("device.description = ", "")
                    .replace('\"', "")
            })
            .filter(|s| !s.is_empty())
            .collect()
    };

    audio_source_combobox.append(Some("default"), &gettext("Default PulseAudio Input Source"));
    for (id, audio_source) in sources_descriptions.iter().enumerate() {
        audio_source_combobox.append(Some(id.to_string().as_str()), audio_source);
    }
    audio_source_combobox.set_active(Some(0));

    // Switchs
    video_switch.set_label(Some(&gettext("Record Video")));
    audio_switch.set_label(Some(&gettext("Record Audio")));
    mouse_switch.set_label(Some(&gettext("Show Mouse")));
    follow_mouse_switch.set_label(Some(&gettext("Follow Mouse")));
    overwrite_switch.set_label(Some(&gettext("Overwrite")));
    video_switch.set_active(config_management::get_bool("default", "videocheck"));
    audio_switch.set_active(config_management::get_bool("default", "audiocheck"));
    mouse_switch.set_active(config_management::get_bool("default", "mousecheck"));
    follow_mouse_switch.set_active(config_management::get_bool("default", "followmousecheck"));
    overwrite_switch.set_active(config_management::get_bool("default", "overwritecheck"));

    let _audio_switch = audio_switch.clone();
    let _mouse_switch = mouse_switch.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    video_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "videocheck", switch.is_active());
        if switch.is_active() {
            _audio_switch.set_active(false);
            _audio_switch.set_sensitive(true);
            _mouse_switch.set_sensitive(true);
        } else {
            _mouse_switch.set_sensitive(false);
            _follow_mouse_switch.set_sensitive(false);
        }
        if !switch.is_active() {
            _audio_switch.set_active(false);
            _audio_switch.set_sensitive(false);
            _mouse_switch.set_active(false);
        }
    });
    let _follow_mouse_switch = follow_mouse_switch.clone();
    mouse_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "mousecheck", switch.is_active());
        if switch.is_active() {
            _follow_mouse_switch.set_sensitive(true);
        } else {
            _follow_mouse_switch.set_active(false);
            _follow_mouse_switch.set_sensitive(false);
        }
    });
    audio_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "audiocheck", switch.is_active());
    });
    follow_mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "followmousecheck", switch.is_active());
    });

    // Buttons
    let mut area_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/screenshot-ui-area-symbolic.svg"));

    if !area_icon_path.exists() {
        area_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
                .add("screenshot-ui-area-symbolic.svg"),
        ))
        .unwrap();
    }

    let mut screen_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/screenshot-ui-display-symbolic.svg"));

    if !screen_icon_path.exists() {
        screen_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
                .add("screenshot-ui-display-symbolic.svg"),
        ))
        .unwrap();
    }

    let mut window_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/screenshot-ui-window-symbolic.svg"));

    if !window_icon_path.exists() {
        window_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
                .add("screenshot-ui-window-symbolic.svg"),
        ))
        .unwrap();
    }

    area_grab_icon.set_from_file(Some(area_icon_path));
    screen_grab_icon.set_from_file(Some(screen_icon_path));
    window_grab_icon.set_from_file(Some(&window_icon_path));

    // Labels
    command_label.set_label(&gettext("Run Command After Recording:"));
    frames_label.set_label(&gettext("Frames:"));
    delay_label.set_label(&gettext("Delay:"));
    audio_source_label.set_label(&gettext("Audio Input Source:"));

    // Spin
    let _frames_spin = frames_spin.to_owned();
    frames_spin.connect_value_changed(move |_| {
        config_management::set(
            "default",
            "frame",
            _frames_spin.value().to_string().as_str(),
        );
    });
    let _delay_spin = delay_spin.to_owned();
    delay_spin.connect_value_changed(move |_| {
        config_management::set(
            "default",
            "delay",
            _delay_spin.value().to_string().as_str(),
        );
    });

    // FileChooser
    let folder_chooser_native = FileChooserNative::new(
            Some("Select Folder"),
            Some(&main_window),
            FileChooserAction::SelectFolder,
            Some("Select"),
            Some("Cancel"),
    );
    folder_chooser_native.set_transient_for(Some(&main_window));
    folder_chooser_native.set_modal(true);
    folder_chooser_native.set_file(&gio::File::for_uri(&config_management::get("default", "folder"))).unwrap();
    let folder_chooser = Some(gio::File::for_uri(&config_management::get("default", "folder"))).unwrap();
    let folder_chooser_name = folder_chooser.basename().unwrap();
    folder_chooser_label.set_label(&folder_chooser_name.to_string_lossy());
    let folder_chooser_icon = config_management::folder_icon(folder_chooser_name.to_str());
    folder_chooser_image.set_icon_name(Some(folder_chooser_icon));
    // Show file chooser dialog
    folder_chooser_button.connect_clicked(glib::clone!(@strong folder_chooser_native => move |_| {
            folder_chooser_native.connect_response(glib::clone!(@strong folder_chooser_native, @strong folder_chooser_label, @strong folder_chooser_image => move |_, response| {
                if response == gtk::ResponseType::Accept {
                    folder_chooser_native.file().unwrap();
                    let folder_chooser = folder_chooser_native.file().unwrap();
                    let folder_chooser_name = folder_chooser.basename().unwrap();
                    folder_chooser_label.set_label(&folder_chooser_name.to_string_lossy());
                    let folder_chooser_icon = config_management::folder_icon(folder_chooser_name.to_str());
                    folder_chooser_image.set_icon_name(Some(folder_chooser_icon));
                };
                folder_chooser_native.hide();
            }));
        folder_chooser_native.show();
    }));

    // --- connections
    // Show dialog window when about button clicked then hide it after close
    let _about_dialog: AboutDialog = about_dialog.to_owned();
    about_button.connect_clicked(move |_| {
        _about_dialog.show();
        _about_dialog.set_hide_on_close(true);
    });

    // Buttons
    let area_capture: Rc<RefCell<area_capture::AreaCapture>> =
        Rc::new(RefCell::new(area_capture::AreaCapture::new()));

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

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    screen_grab_button.connect_clicked(move |_| {
        _area_chooser_window.hide();
        _area_capture.borrow_mut().reset();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    window_grab_button.connect_clicked(move |_| {
        _area_chooser_window.hide();
        _area_capture.borrow_mut().get_area();
    });

    // Init record struct
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        filename: (folder_chooser_native, filename_entry, format_chooser_combobox),
        record_video: video_switch,
        record_audio: audio_switch,
        audio_id: audio_source_combobox,
        record_mouse: mouse_switch,
        follow_mouse: follow_mouse_switch,
        record_frames: frames_spin,
        record_delay: delay_spin,
        command: command_entry,
        video_process_id: None,
        audio_process_id: None,
        saved_filename: None,
        unbound: None,
        progress_widget: ProgressWidget::new(progress_dialog, progressbar),
        window: main_window.clone(),
        overwrite: overwrite_switch,
    }));

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let _stop_button = stop_button.clone();
    let _record_button = record_button.clone();
    record_button.connect_clicked(move |_| {
        let _area_capture = area_capture.borrow_mut();
        match _ffmpeg_record_interface.borrow_mut().start_record(
            _area_capture.x,
            _area_capture.y,
            _area_capture.width,
            _area_capture.height,
        ) {
            (None, None) => {
                    // Do nothing if the start_record function return nothing
                    }
            _ => {
                _record_button.hide();
                _stop_button.show();
            }
        }
    });

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let _stop_button = stop_button.clone();
    let _play_button = play_button.clone();
    stop_button.connect_clicked(move |_| {
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        record_button.show();
        _stop_button.hide();
        _play_button.show();
    });

    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    play_button.connect_clicked(move |_| {
        _ffmpeg_record_interface.borrow_mut().clone().play_record();
    });

    // About Dialog
    let mut about_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/blue-recorder@x96.png"));

    if !about_icon_path.exists() {
        about_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
                .add("blue-recorder@x96.png"),
        ))
        .unwrap();
    }

    let logo = Image::from_file(&about_icon_path.to_str().unwrap());
    about_dialog.set_transient_for(Some(&main_window));
    about_dialog.set_program_name(Some(&gettext("Blue Recorder")));
    about_dialog.set_version(Some("0.2.0"));
    about_dialog.set_copyright(Some("© 2021 Salem Yaslem"));
    about_dialog.set_wrap_license(true);
    about_dialog.set_license(Some("Blue Recorder is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\nBlue Recorder is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.\n\nSee the GNU General Public License for more details. You should have received a copy of the GNU General Public License along with Blue Recorder. If not, see <http://www.gnu.org/licenses/>."));
    about_dialog.set_comments(Some(&gettext(
        "A simple screen recorder for Linux desktop. Supports Waylan_windowd & Xorg.",
    )));
    about_dialog.set_authors(&[
        "Salem Yaslem <s@sy.sa>",
        "M.Hanny Sabbagh <mhsabbagh@outlook.com>",
        "Alessandro Toia <gort818@gmail.com>",
        "Suliman Altassan <suliman@dismail.de>",
        "O.Chibani <11yzyv86j@relay.firefox.com>",
        "Patreon Supporters: Ahmad Gharib, Medium,\nWilliam Grunow, Alex Benishek.",
    ]);
    about_dialog.set_artists(&["Mustapha Assabar", "Abdullah Al-Baroty <albaroty@gmail.com>"]);
    about_dialog.set_website(Some("https://github.com/xlmnxp/blue-recorder/"));
    about_dialog.set_logo_icon_name(Some("blue-recorder"));
    about_dialog.set_logo(logo.paintable().as_ref());
    about_dialog.set_modal(true);

    // Windows
    // Hide area chooser after it deleted.
    let _area_chooser_window = area_chooser_window.clone();
    area_chooser_window.connect_close_request (move |_| {
        _area_chooser_window.hide();
        gtk::Inhibit(true)
    });

   // Close the application when main window destroy
    main_window.connect_destroy(move |main_window| {
        let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
        // Stop recording before close the application
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        main_window.close();
    });

    // Apply css
    let provider = CssProvider::new();
    provider
        .load_from_data(include_str!("styles/global.css").as_bytes());
    gtk::StyleContext::add_provider_for_display(
        &area_chooser_window.display(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    fn is_wayland() -> bool {
        std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .eq_ignore_ascii_case("wayland")
    }

    main_window.show();
}
