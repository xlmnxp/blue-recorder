extern crate gdk;
extern crate gio;
extern crate gtk;
mod area_capture;
mod config_management;
mod ffmpeg_interface;
mod timer;
mod wayland_record;
mod utils;

use ffmpeg_interface::Ffmpeg;
use fluent_bundle::bundle::FluentBundle;
use fluent_bundle::FluentResource;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
    AboutDialog, Application, Builder, Button, CheckButton, ComboBoxText, CssProvider, Entry,
    FileChooserAction, FileChooserNative, Image, Label, SpinButton,
    ToggleButton, Window,
};
use utils::is_wayland;
use std::cell::RefCell;
use std::ops::Add;
use std::path::Path;
use std::rc::Rc;
use timer::{recording_delay, start_timer, stop_timer};
use wayland_record::WaylandRecorder;
use cpal::traits::{DeviceTrait, HostTrait};


#[async_std::main]
async fn main() {
    // Create new application
    let application = Application::new(None, Default::default());
    application.connect_activate(build_ui);
    application.run();
}

pub fn build_ui(application: &Application) {
    gtk::init().expect("Failed to initialize GTK.");

    // UI source
    let ui_src = include_str!("../interfaces/main.ui").to_string();
    let builder: Builder = Builder::from_string(ui_src.as_str());

    // Init audio source
    let host_audio_device = cpal::default_host();

    // Translate
    let mut ftl_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }.join(Path::new("locales"));
    if !ftl_path.exists() {
        ftl_path = std::fs::canonicalize(Path::new(
            &std::env::var("LC_DIR").unwrap_or_else(|_| String::from("locales")),
        )).unwrap();
    }
    let supported_lang: Vec<String> = std::fs::read_dir(&ftl_path)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            path.file_stem().unwrap().to_string_lossy().to_string()
        }).collect();
    let mut locale = std::env::var("LANG").unwrap_or("en_US".to_string());
    if !supported_lang.contains(&locale) {
        locale = locale.split('_').next().unwrap().to_string();
        if !supported_lang.contains(&locale) {
            locale = String::from("en_US");
        }
    }
    let ftl_file = std::fs::read_to_string(
        format!("{}/{}.ftl", ftl_path.to_str().unwrap(),locale.split('.').next().unwrap())
    ).unwrap();
    let res = FluentResource::try_new(ftl_file).unwrap();
    let mut bundle = FluentBundle::default();
    bundle.add_resource(res).expect("Failed to add localization resources to the bundle.");

    // Config initialize
    config_management::initialize();

    // Get Objects from UI
    let area_apply_label: Label = builder.object("area_apply").unwrap();
    let area_chooser_window: Window = builder.object("area_chooser_window").unwrap();
    let area_grab_button: ToggleButton = builder.object("area_grab_button").unwrap();
    let area_grab_icon: Image = builder.object("area_grab_icon").unwrap();
    let area_grab_label: Label = builder.object("area_grab_label").unwrap();
    let area_set_button: Button = builder.object("area_set_button").unwrap();
    let about_button: Button = builder.object("aboutbutton").unwrap();
    let about_dialog: AboutDialog = builder.object("about_dialog").unwrap();
    let audio_bitrate_label: Label = builder.object("audio_bitrate_label").unwrap();
    let audio_bitrate_spin: SpinButton = builder.object("audio_bitrate").unwrap();
    let audio_source_combobox: ComboBoxText = builder.object("audiosource").unwrap();
    let audio_source_label: Label = builder.object("audio_source_label").unwrap();
    let audio_switch: CheckButton = builder.object("audioswitch").unwrap();
    let command_entry: Entry = builder.object("command").unwrap();
    let command_label: Label = builder.object("command_label").unwrap();
    let delay_label: Label = builder.object("delay_label").unwrap();
    let delay_spin: SpinButton = builder.object("delay").unwrap();
    let delay_window: Window = builder.object("delay_window").unwrap();
    let delay_window_button: ToggleButton = builder.object("delay_window_stopbutton").unwrap();
    let delay_window_label: Label = builder.object("delay_window_label").unwrap();
    let delay_window_title: Label = builder.object("delay_window_title").unwrap();
    let filename_entry: Entry = builder.object("filename").unwrap();
    let folder_chooser_button: Button = builder.object("folder_chooser").unwrap();
    let folder_chooser_image: Image = builder.object("folder_chooser_image").unwrap();
    let folder_chooser_label: Label = builder.object("folder_chooser_label").unwrap();
    let follow_mouse_switch: CheckButton = builder.object("followmouseswitch").unwrap();
    let format_chooser_combobox: ComboBoxText = builder.object("comboboxtext1").unwrap();
    let frames_label: Label = builder.object("frames_label").unwrap();
    let frames_spin: SpinButton = builder.object("frames").unwrap();
    let hide_switch: CheckButton = builder.object("hideswitch").unwrap();
    let main_window: Window = builder.object("main_window").unwrap();
    let mouse_switch: CheckButton = builder.object("mouseswitch").unwrap();
    let play_button: Button = builder.object("playbutton").unwrap();
    let record_button: Button = builder.object("recordbutton").unwrap();
    let record_label: Label = builder.object("record_label").unwrap();
    let record_time_label: Label = builder.object("record_time_label").unwrap();
    let screen_grab_button: ToggleButton = builder.object("screen_grab_button").unwrap();
    let screen_grab_icon: Image = builder.object("screen_grab_icon").unwrap();
    let screen_grab_label: Label = builder.object("screen_grab_label").unwrap();
    let speaker_switch: CheckButton = builder.object("speakerswitch").unwrap();
    let stop_button: Button = builder.object("stopbutton").unwrap();
    let stop_label: Label = builder.object("stop_label").unwrap();
    let video_bitrate_label: Label = builder.object("video_bitrate_label").unwrap();
    let video_bitrate_spin: SpinButton = builder.object("video_bitrate").unwrap();
    let video_switch: CheckButton = builder.object("videoswitch").unwrap();
    let window_grab_button: ToggleButton = builder.object("window_grab_button").unwrap();
    let window_grab_icon: Image = builder.object("window_grab_icon").unwrap();
    let window_grab_label: Label = builder.object("window_grab_label").unwrap();

    // --- default properties
    // Windows
    main_window.set_title(Some(&bundle.format_pattern(bundle.get_message("blue-recorder").unwrap()
                                                      .value().unwrap(), None, &mut vec![]).to_string()));
    main_window.set_application(Some(application));
    area_chooser_window.set_title(Some(&bundle.format_pattern(bundle.get_message("area-chooser").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string())); // Title is hidden
    
    // Hide stop & play buttons
    stop_button.hide();
    play_button.hide();

    // Toggle button
    config_management::set("default", "mode", "screen");
    screen_grab_button.set_active(true);

    // Comboboxs tooltip
    audio_source_combobox.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("audio-source-tooltip").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    format_chooser_combobox.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("format-tooltip").unwrap()
                                                                         .value().unwrap(), None, &mut vec![]).to_string()));

    area_grab_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("area-tooltip").unwrap()
                                                                  .value().unwrap(), None, &mut vec![]).to_string()));
    // Temporary solution
    if is_wayland() {
        // Disabled for the tooltip
        //area_grab_button.set_can_focus(false);
        //area_grab_button.set_can_target(false);
        //area_grab_button.add_css_class("disabled");
        area_grab_button.set_sensitive(false);
        // Hide window grab button in Wayland
        area_grab_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("wayland-tooltip").unwrap()
                                                                      .value().unwrap(), None, &mut vec![]).to_string()));
    }

    // Entries
    filename_entry.set_placeholder_text(Some(&bundle.format_pattern(bundle.get_message("file-name").unwrap()
                                                                    .value().unwrap(), None, &mut vec![]).to_string()));
    command_entry.set_placeholder_text(Some(&bundle.format_pattern(bundle.get_message("default-command").unwrap()
                                                                   .value().unwrap(), None, &mut vec![]).to_string()));
    filename_entry.set_text(&config_management::get("default", "filename"));
    command_entry.set_text(&config_management::get("default", "command"));

    // Format combobox
    format_chooser_combobox.append(Some("mp4"), &bundle.format_pattern(bundle.get_message("mp4-format").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.append(
        Some("mkv"),
        &bundle.format_pattern(bundle.get_message("mkv-format").unwrap()
                               .value().unwrap(), None, &mut vec![]).to_string(),
    );
    format_chooser_combobox.append(Some("webm"), &bundle.format_pattern(bundle.get_message("webm-format").unwrap()
                                                                        .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.append(Some("gif"), &bundle.format_pattern(bundle.get_message("gif-format").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.append(Some("avi"), &bundle.format_pattern(bundle.get_message("avi-format").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.append(Some("wmv"), &bundle.format_pattern(bundle.get_message("wmv-format").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.append(Some("nut"), &bundle.format_pattern(bundle.get_message("nut-format").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string());
    format_chooser_combobox.set_active(Some(config_management::get("default", "format").parse::<u32>().unwrap()));

    // Get audio sources
    let input_device = host_audio_device.input_devices().unwrap();
    let sources_descriptions: Vec<String> = input_device
        .filter_map(|device| device.name().ok())
        .collect();

    audio_source_combobox.append(Some("default"), &bundle.format_pattern(bundle.get_message("audio-input").unwrap()
                                                                         .value().unwrap(), None, &mut vec![]).to_string());
    for (id, audio_source) in sources_descriptions.iter().enumerate() {
        audio_source_combobox.append(Some(id.to_string().as_str()), audio_source);
    }
    audio_source_combobox.set_active(Some(0));

    // Switchs
    video_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("video-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    video_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("record-video").unwrap()
                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    audio_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("audio-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    audio_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("record-audio").unwrap()
                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    mouse_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("mouse-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    mouse_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("show-mouse").unwrap()
                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    follow_mouse_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("follow-mouse-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    follow_mouse_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("follow-mouse").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    hide_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("hide-tooltip").unwrap()
                                                             .value().unwrap(), None, &mut vec![]).to_string()));
    hide_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("auto-hide").unwrap()
                                                      .value().unwrap(), None, &mut vec![]).to_string()));
    speaker_switch.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("speaker-tooltip").unwrap()
                                                             .value().unwrap(), None, &mut vec![]).to_string()));
    speaker_switch.set_label(Some(&bundle.format_pattern(bundle.get_message("record-speaker").unwrap()
                                                      .value().unwrap(), None, &mut vec![]).to_string()));
    video_switch.set_active(config_management::get_bool("default", "videocheck"));
    audio_switch.set_active(config_management::get_bool("default", "audiocheck"));
    mouse_switch.set_active(config_management::get_bool("default", "mousecheck"));
    follow_mouse_switch.set_active(config_management::get_bool("default", "followmousecheck"));
    hide_switch.set_active(config_management::get_bool("default", "hidecheck"));
    speaker_switch.set_active(config_management::get_bool("default", "speakercheck"));

    let _video_switch = video_switch.clone();
    let _audio_switch = audio_switch.clone();
    let _mouse_switch = mouse_switch.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    video_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "videocheck", switch.is_active());
        if switch.is_active() {
            _mouse_switch.set_sensitive(true);
            _follow_mouse_switch.set_sensitive(true);
        } else {
            _mouse_switch.set_sensitive(false);
            _follow_mouse_switch.set_sensitive(false);
            _audio_switch.set_active(true);
            _audio_switch.set_sensitive(true);
            _mouse_switch.set_active(false);
            _follow_mouse_switch.set_active(false);
        }
    });
    let _follow_mouse_switch = follow_mouse_switch.clone();
    mouse_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "mousecheck", switch.is_active());
    });
    let _mouse_switch = mouse_switch.clone();
    audio_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "audiocheck", switch.is_active());
        if !switch.is_active() && !_video_switch.is_active() {
            _video_switch.set_active(true);
            _mouse_switch.set_sensitive(true);
        }
    });
    follow_mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "followmousecheck", switch.is_active());
    });
    hide_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "hidecheck", switch.is_active());
    });
    speaker_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "speakercheck", switch.is_active());
    });

    match dark_light::detect() {
        // Dark mode
        dark_light::Mode::Dark => {
            // Buttons
            let mut area_icon_path = {
                let mut current_exec_dir = std::env::current_exe().unwrap();
                current_exec_dir.pop();
                current_exec_dir
            }
            .join(Path::new("data/screenshot-ui-area-symbolic-white.svg"));

            if !area_icon_path.exists() {
                area_icon_path = std::fs::canonicalize(Path::new(
                    &std::env::var("DATA_DIR")
                        .unwrap_or_else(|_| String::from("data/"))
                        .add("screenshot-ui-area-symbolic-white.svg"),
                ))
                .unwrap();
            }

            let mut screen_icon_path = {
                let mut current_exec_dir = std::env::current_exe().unwrap();
                current_exec_dir.pop();
                current_exec_dir
            }
            .join(Path::new("data/screenshot-ui-display-symbolic-white.svg"));

            if !screen_icon_path.exists() {
                screen_icon_path = std::fs::canonicalize(Path::new(
                    &std::env::var("DATA_DIR")
                        .unwrap_or_else(|_| String::from("data/"))
                        .add("screenshot-ui-display-symbolic-white.svg"),
                ))
                .unwrap();
            }

            let mut window_icon_path = {
                let mut current_exec_dir = std::env::current_exe().unwrap();
                current_exec_dir.pop();
                current_exec_dir
            }
            .join(Path::new("data/screenshot-ui-window-symbolic-white.svg"));

            if !window_icon_path.exists() {
                window_icon_path = std::fs::canonicalize(Path::new(
                    &std::env::var("DATA_DIR")
                        .unwrap_or_else(|_| String::from("data/"))
                        .add("screenshot-ui-window-symbolic-white.svg"),
                ))
                .unwrap();
            }

            area_grab_icon.set_from_file(Some(area_icon_path));
            screen_grab_icon.set_from_file(Some(screen_icon_path));
            window_grab_icon.set_from_file(Some(&window_icon_path));        
        }
        // any theme
        _ => {
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
        }
    }

    // Spin
    frames_spin.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("frames-tooltip").unwrap()
                                                            .value().unwrap(), None, &mut vec![]).to_string()));
    delay_spin.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("delay-tooltip").unwrap()
                                                             .value().unwrap(), None, &mut vec![]).to_string()));
    video_bitrate_spin.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("video-bitrate-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    audio_bitrate_spin.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("audio-bitrate-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    frames_spin.set_value(
        config_management::get("default",
                               &format!
                               ("frame-{}",
                                &format_chooser_combobox.active().unwrap().to_string()))
            .parse::<f64>()
            .unwrap(),
    );
    delay_spin.set_value(
        config_management::get("default", "delay")
            .parse::<f64>()
            .unwrap(),
    );
    video_bitrate_spin.set_value(
        config_management::get("default",
                               &format!
                               ("videobitrate-{}",
                                &format_chooser_combobox.active().unwrap().to_string()))
            .parse::<f64>()
            .unwrap(),
    );
    audio_bitrate_spin.set_value(
        config_management::get("default", "audiobitrate")
            .parse::<f64>()
            .unwrap(),
    );

    let _format_chooser_combobox = format_chooser_combobox.clone();
    let _frames_spin = frames_spin.clone();
    let _video_bitrate_spin = video_bitrate_spin.clone();
    format_chooser_combobox.connect_changed(move |_| {
        let format_chooser_combobox = _format_chooser_combobox.clone();
        if _format_chooser_combobox.active_text().is_some() {
            config_management::set(
                "default",
                "format",
                &_format_chooser_combobox.active().unwrap().to_string(),
            );
            _frames_spin.set_value(
                config_management::get("default",
                                       &format!
                                       ("frame-{}",
                                        &format_chooser_combobox.active().unwrap().to_string()))
                    .parse::<f64>()
                    .unwrap(),
            );
            _video_bitrate_spin.set_value(
                config_management::get("default",
                                       &format!
                                       ("videobitrate-{}",
                                        &format_chooser_combobox.active().unwrap().to_string()))
                    .parse::<f64>()
                    .unwrap(),
            );
        }
    });

    let _frames_spin = frames_spin.to_owned();
    let _format_chooser_combobox = format_chooser_combobox.clone();
    frames_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               &format!
                               ("frame-{}",
                                &_format_chooser_combobox.active().unwrap().to_string()),
                               _frames_spin.value().to_string().as_str());
    });
    let _delay_spin = delay_spin.to_owned();
    delay_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               "delay",
                               _delay_spin.value().to_string().as_str());
    });
    let _video_bitrate_spin = video_bitrate_spin.to_owned();
    let _format_chooser_combobox = format_chooser_combobox.clone();
    video_bitrate_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               &format!
                               ("videobitrate-{}",
                                &_format_chooser_combobox.active().unwrap().to_string()),
                               _video_bitrate_spin.value().to_string().as_str());
     });
    let _audio_bitrate_spin = audio_bitrate_spin.to_owned();
    audio_bitrate_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               "audio_bitrate",
                               _audio_bitrate_spin.value().to_string().as_str());
    });

    // Labels
    command_label.set_label(&bundle.format_pattern(bundle.get_message("run-command").unwrap()
                                                   .value().unwrap(), None, &mut vec![]).to_string());
    frames_label.set_label(&bundle.format_pattern(bundle.get_message("frames").unwrap()
                                                  .value().unwrap(), None, &mut vec![]).to_string());
    delay_label.set_label(&bundle.format_pattern(bundle.get_message("delay").unwrap()
                                                 .value().unwrap(), None, &mut vec![]).to_string());
    video_bitrate_label.set_label(&bundle.format_pattern(bundle.get_message("video-bitrate").unwrap()
                                                 .value().unwrap(), None, &mut vec![]).to_string());
    audio_bitrate_label.set_label(&bundle.format_pattern(bundle.get_message("audio-bitrate").unwrap()
                                                 .value().unwrap(), None, &mut vec![]).to_string());
    audio_source_label.set_label(&bundle.format_pattern(bundle.get_message("audio-source").unwrap()
                                                        .value().unwrap(), None, &mut vec![]).to_string());

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
    folder_chooser_native
        .set_file(&gio::File::for_uri(&config_management::get(
            "default", "folder",
        )))
        .unwrap();
    let folder_chooser = Some(gio::File::for_uri(&config_management::get(
        "default", "folder",
    )))
    .unwrap();
    let folder_chooser_name = folder_chooser.basename().unwrap();
    folder_chooser_label.set_label(&folder_chooser_name.to_string_lossy());
    let folder_chooser_icon = config_management::folder_icon(folder_chooser_name.to_str());
    folder_chooser_image.set_icon_name(Some(folder_chooser_icon));
    folder_chooser_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("folder-tooltip").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string()));
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
    about_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("about-tooltip").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    about_button.set_label(&bundle.format_pattern(bundle.get_message("about").unwrap()
                                                  .value().unwrap(), None, &mut vec![]).to_string());
    about_button.connect_clicked(move |_| {
        _about_dialog.show();
        _about_dialog.set_hide_on_close(true);
    });

    // Buttons
    let area_capture: Rc<RefCell<area_capture::AreaCapture>> =
        Rc::new(RefCell::new(area_capture::AreaCapture::new()));

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    area_grab_label.set_label(&bundle.format_pattern(bundle.get_message("select-area").unwrap()
                                                  .value().unwrap(), None, &mut vec![]).to_string());
    area_grab_button.connect_clicked(move |_| {
        config_management::set("default", "mode", "area");
        _area_chooser_window.show();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    area_apply_label.set_label(&bundle.format_pattern(bundle.get_message("apply").unwrap()
                                                      .value().unwrap(), None, &mut vec![]).to_string());
    area_set_button.connect_clicked(move |_| {
        _area_capture
            .borrow_mut()
            .get_window_by_name(_area_chooser_window.title().unwrap().as_str());
        _area_chooser_window.hide();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    let record_window: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let window_grab_button_record_window: Rc<RefCell<bool>> = record_window.clone();
    let screen_grab_button_record_window: Rc<RefCell<bool>> = record_window.clone();
    screen_grab_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("screen-tooltip").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    screen_grab_label.set_label(&bundle.format_pattern(bundle.get_message("select-screen").unwrap()
                                                      .value().unwrap(), None, &mut vec![]).to_string());
    screen_grab_button.connect_clicked(move |_| {
        config_management::set("default", "mode", "screen");
        screen_grab_button_record_window.replace(false);
        _area_chooser_window.hide();
        _area_capture.borrow_mut().reset();
    });

    let _area_chooser_window: Window = area_chooser_window.clone();
    let mut _area_capture: Rc<RefCell<area_capture::AreaCapture>> = area_capture.clone();
    window_grab_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("window-tooltip").unwrap()
                                                                       .value().unwrap(), None, &mut vec![]).to_string()));
    window_grab_label.set_label(&bundle.format_pattern(bundle.get_message("select-window").unwrap()
                                                       .value().unwrap(), None, &mut vec![]).to_string());
    window_grab_button.connect_clicked(move |_| {
        config_management::set("default", "mode", "window");
        _area_chooser_window.hide();
        if is_wayland() {
            window_grab_button_record_window.replace(true);
        } else {
            _area_capture.borrow_mut().get_area();
        }
    });

    let _delay_spin = delay_spin.clone();

    let main_context = glib::MainContext::default();
    let wayland_record = main_context.block_on(WaylandRecorder::new());
    let bundle_msg = bundle.format_pattern(bundle.get_message("already-exist").unwrap()
                                            .value().unwrap(), None, &mut vec![]).to_string();
    // Init record struct
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        filename: (
            folder_chooser_native,
            filename_entry,
            format_chooser_combobox,
        ),
        record_video: video_switch,
        record_audio: audio_switch,
        audio_id: audio_source_combobox,
        record_mouse: mouse_switch,
        follow_mouse: follow_mouse_switch,
        record_frames: frames_spin,
        command: command_entry,
        video_process: None,
        audio_process: None,
        saved_filename: None,
        height: None,
        unbound: None,
        window: main_window.clone(),
        record_delay: delay_spin,
        record_wayland: wayland_record,
        record_window,
        main_context,
        temp_video_filename: String::new(),
        bundle: bundle_msg,
        video_record_bitrate: video_bitrate_spin,
        audio_record_bitrate: audio_bitrate_spin,
    }));

    // Record Button
    let _delay_window = delay_window.clone();
    let _delay_window_button = delay_window_button.clone();
    let _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let _main_window = main_window.clone();
    let _play_button = play_button.clone();
    let _record_button = record_button.clone();
    let _record_time_label = record_time_label.clone();
    let _stop_button = stop_button.clone();
    record_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("record-tooltip").unwrap()
                                                               .value().unwrap(), None, &mut vec![]).to_string()));
    record_label.set_label(&bundle.format_pattern(bundle.get_message("record").unwrap()
                                                   .value().unwrap(), None, &mut vec![]).to_string());
    delay_window_title.set_label(&bundle.format_pattern(bundle.get_message("delay-title").unwrap()
                                                        .value().unwrap(), None, &mut vec![]).to_string());
    delay_window_button.set_label(&bundle.format_pattern(bundle.get_message("delay-window-stop").unwrap()
                                                        .value().unwrap(), None, &mut vec![]).to_string());
    record_button.connect_clicked(move |_| {
        _delay_window_button.set_active(false);
        if _delay_spin.value() as u64 > 0 {
            recording_delay(
                _delay_spin.clone(),
                _delay_spin.value() as u64,
                delay_window.clone(),
                _delay_window_button.clone(),
                delay_window_label.clone(),
                _record_button.clone(),
            );
        } else if _delay_spin.value() as u64 == 0 {
            let _area_capture = area_capture.borrow_mut();
            match _ffmpeg_record_interface.borrow_mut().start_record(
                _area_capture.x,
                _area_capture.y,
                _area_capture.width,
                _area_capture.height,
            ) {
                None => {
                    // Do nothing if the start_record function return nothing
                }
                _ => {
                    start_timer(record_time_label.clone());
                    record_time_label.set_visible(true);
                    if hide_switch.is_active() {
                        _main_window.minimize();
                    }
                    _play_button.hide();
                    _record_button.hide();
                    _stop_button.show();
                }
            }
        }
    });

    // Stop Record Button
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    let _play_button = play_button.clone();
    let _stop_button = stop_button.clone();
    stop_button.set_tooltip_text(Some(&bundle.format_pattern(bundle.get_message("stop-tooltip").unwrap()
                                                             .value().unwrap(), None, &mut vec![]).to_string()));
    stop_label.set_label(&bundle.format_pattern(bundle.get_message("stop-recording").unwrap()
                                                .value().unwrap(), None, &mut vec![]).to_string());
    stop_button.connect_clicked(move |_| {
        _record_time_label.set_visible(false);
        stop_timer(_record_time_label.clone());
        _ffmpeg_record_interface.borrow_mut().clone().stop_record();
        record_button.show();
        _stop_button.hide();
        _play_button.show();
    });

    // Delay Window Button
    let _delay_window_button = delay_window_button.clone();
    delay_window_button.connect_clicked(move |_| {});

    // Play Button
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
    about_dialog.set_program_name(Some(&bundle.format_pattern(bundle.get_message("blue-recorder").unwrap()
                                                              .value().unwrap(), None, &mut vec![]).to_string()));
    about_dialog.set_version(Some("0.3.0"));
    about_dialog.set_copyright(Some(&bundle.format_pattern(bundle.get_message("copy-right").unwrap()
                                                           .value().unwrap(), None, &mut vec![]).to_string()));
    about_dialog.set_wrap_license(true);
    about_dialog.set_license(Some(&bundle.format_pattern(bundle.get_message("license").unwrap()
                                                         .value().unwrap(), None, &mut vec![]).to_string()));
    about_dialog.set_comments(Some(&bundle.format_pattern(bundle.get_message("dialog-comment").unwrap()
                                                          .value().unwrap(), None, &mut vec![]).to_string()));
    // Authors
    about_dialog.add_credit_section(
        &bundle.format_pattern(bundle.get_message("authors").unwrap()
                               .value().unwrap(), None, &mut vec![]).to_string(),
        &[&bundle.format_pattern(bundle.get_message("address-abdullah-al-baroty").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-alessandro-toia").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-chibani").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-hamir-mahal").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-hanny-sabbagh").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-salem-yaslem").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-suliman-altassan").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
        ]
    );
    // Patreon suppoters
    about_dialog.add_credit_section(
        &bundle.format_pattern(bundle.get_message("patreon").unwrap()
                               .value().unwrap(), None, &mut vec![]).to_string(),
        &[&bundle.format_pattern(bundle.get_message("address-ahmad-gharib").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-medium").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-william-grunow").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-alex-benishek").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
        ]
    );
    // Designers
    about_dialog.add_credit_section(
        &bundle.format_pattern(bundle.get_message("design").unwrap()
                               .value().unwrap(), None, &mut vec![]).to_string(),
        &[&bundle.format_pattern(bundle.get_message("address-abdullah-al-baroty").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-mustapha-assabar").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
        ]
    );
    // Translators
    about_dialog.add_credit_section(
        &bundle.format_pattern(bundle.get_message("translate").unwrap()
                               .value().unwrap(), None, &mut vec![]).to_string(),
        &[&bundle.format_pattern(bundle.get_message("address-ake-engelbrektson").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-amerey").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-gmou3").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-larry-wei").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-mark-wagie").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-albanobattistella").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
          &bundle.format_pattern(bundle.get_message("address-mr-Narsus").unwrap()
                                 .value().unwrap(), None, &mut vec![]).to_string(),
       ]
    );
    about_dialog.set_website(Some("https://github.com/xlmnxp/blue-recorder/"));
    about_dialog.set_website_label(&bundle.format_pattern(bundle.get_message("website").unwrap()
                                                          .value().unwrap(), None, &mut vec![]).to_string());
    about_dialog.set_logo_icon_name(Some("blue-recorder"));
    about_dialog.set_logo(logo.paintable().as_ref());
    about_dialog.set_modal(true);

    // Windows
    // Hide area chooser after it deleted.
    let _area_chooser_window = area_chooser_window.clone();
    area_chooser_window.connect_close_request(move |_| {
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

    // Apply CSS
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("styles/global.css").as_bytes());
    gtk::StyleContext::add_provider_for_display(
        &area_chooser_window.display(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    main_window.show();
}
