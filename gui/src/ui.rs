use adw::{Application, Window};
use adw::gio::File;
use adw::gtk::{AboutDialog, Builder, Button, CheckButton, ComboBoxText, CssProvider, Entry, Expander, FileChooserNative,
               FileChooserAction, Image, Label, MessageDialog, SpinButton, TextBuffer, TextView, ToggleButton};
use adw::prelude::*;
use anyhow::Result;
#[cfg(any(target_os = "freebsd", target_os = "linux"))]
use blue_recorder_core::ffmpeg_linux::Ffmpeg;
#[cfg(target_os = "windows")]
use blue_recorder_core::ffmpeg_windows::Ffmpeg;
use blue_recorder_core::utils::{is_wayland, play_record, RecordMode};
use cpal::traits::{DeviceTrait, HostTrait};
use std::cell::RefCell;
use std::ops::Add;
use std::path::Path;
use std::rc::Rc;

use crate::{area_capture, config_management, fluent::get_bundle};
use crate::timer::{recording_delay, start_timer, stop_timer};

pub fn run_ui(application: &Application) {
    // Error dialog
    let error_dialog_ui_src = include_str!("../interfaces/error_dialog.ui").to_string();
    let builder: Builder = Builder::from_string(error_dialog_ui_src.as_str());
    let error_dialog: MessageDialog = builder.object("error_dialog").unwrap();
    let error_dialog_button: Button = builder.object("error_button").unwrap();
    let error_dialog_label: Label = builder.object("error_text").unwrap();
    let error_expander: Expander = builder.object("error_expander").unwrap();
    let error_expander_label: Label = builder.object("expander_label").unwrap();
    let error_message: TextView = builder.object("error_details").unwrap();
    error_dialog_button.set_label(&get_bundle("close", None));
    error_expander_label.set_label(&get_bundle("details-button", None));
    error_dialog_label.set_label(&get_bundle("some-error", None));
    error_dialog.set_title(Some(&get_bundle("error-title", None)));
    let _error_dialog = error_dialog.clone();
    let _error_expander = error_expander.clone();
    error_dialog_button.connect_clicked(move |_| {
        _error_expander.set_expanded(false);
        _error_dialog.set_hide_on_close(true);
        _error_dialog.close();
    });

    match build_ui(application, error_dialog.clone(), error_message.clone()) {
        Ok(_) => {
            // Continue
        },
        Err(error) => {
            let text_buffer = TextBuffer::new(None);
            text_buffer.set_text(&error.to_string());
            error_message.set_buffer(Some(&text_buffer));
            error_dialog.show();
            error_dialog.set_hide_on_close(true);
        }
    }
}

fn build_ui(application: &Application, error_dialog: MessageDialog, error_message: TextView) -> Result<()> {
    // Init audio source
    let host_audio_device = cpal::default_host();

    // Config initialize
    config_management::initialize();

    // UI source
    let about_dialog_ui_src = include_str!("../interfaces/about_dialog.ui").to_string();
    let area_selection_ui_src = include_str!("../interfaces/area_selection.ui").to_string();
    let delay_ui_src = include_str!("../interfaces/delay.ui").to_string();
    let main_ui_src = include_str!("../interfaces/main.ui").to_string();
    let select_window_ui_src = include_str!("../interfaces/select_window.ui").to_string();

    let builder: Builder = Builder::from_string(main_ui_src.as_str());
    builder.add_from_string(about_dialog_ui_src.as_str()).unwrap();
    builder.add_from_string(area_selection_ui_src.as_str()).unwrap();
    builder.add_from_string(delay_ui_src.as_str()).unwrap();
    builder.add_from_string(select_window_ui_src.as_str()).unwrap();

    // Get Objects from UI
    let area_apply_label: Label = builder.object("area_apply").unwrap();
    let area_chooser_window: Window = builder.object("area_chooser_window").unwrap();
    let area_grab_button: ToggleButton = builder.object("area_grab_button").unwrap();
    let area_grab_icon: Image = builder.object("area_grab_icon").unwrap();
    let area_grab_label: Label = builder.object("area_grab_label").unwrap();
    let area_set_button: Button = builder.object("area_set_button").unwrap();
    let area_size_bottom_label: Label = builder.object("area_size_bottom").unwrap();
    let area_size_top_label: Label = builder.object("area_size_top").unwrap();
    let area_switch: CheckButton = builder.object("areaswitch").unwrap();
    let about_button: Button = builder.object("aboutbutton").unwrap();
    let about_dialog: AboutDialog = builder.object("about_dialog").unwrap();
    let audio_bitrate_label: Label = builder.object("audio_bitrate_label").unwrap();
    let audio_bitrate_spin: SpinButton = builder.object("audio_bitrate").unwrap();
    let audio_source_combobox: ComboBoxText = builder.object("audiosource").unwrap();
    let audio_source_label: Label = builder.object("audio_source_label").unwrap();
    let audio_input_switch: CheckButton = builder.object("audio_input_switch").unwrap();
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
    let tray_switch: CheckButton = builder.object("trayswitch").unwrap();
    let screen_grab_button: ToggleButton = builder.object("screen_grab_button").unwrap();
    let screen_grab_icon: Image = builder.object("screen_grab_icon").unwrap();
    let screen_grab_label: Label = builder.object("screen_grab_label").unwrap();
    let select_window: Window = builder.object("select_window").unwrap();
    #[cfg(target_os = "windows")]
    let select_window_label: Label = builder.object("select_window_label").unwrap();
    let audio_output_switch: CheckButton = builder.object("speakerswitch").unwrap();
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
    area_chooser_window.set_title(Some(&get_bundle("area-chooser", None))); // Title is hidden
    error_dialog.set_transient_for(Some(&main_window));
    select_window.set_transient_for(Some(&main_window));
    main_window.set_application(Some(application));
    main_window.set_title(Some(&get_bundle("blue-recorder", None)));

    // Hide stop & play buttons
    play_button.hide();
    stop_button.hide();

    // Disable show area check button
    if !area_grab_button.is_active() {
        area_switch.set_active(false);
        area_switch.set_sensitive(false);
    }

    // Toggle button
    config_management::set("default", "mode", "screen");
    screen_grab_button.set_active(true);

    // Comboboxs tooltip
    area_grab_button.set_tooltip_text(Some(&get_bundle("area-tooltip", None)));
    audio_source_combobox.set_tooltip_text(Some(&get_bundle("audio-source-tooltip", None)));
    format_chooser_combobox.set_tooltip_text(Some(&get_bundle("format-tooltip", None)));

    // Temporary solution
    if is_wayland() {
        // Hide window grab button in Wayland
        area_grab_button.set_sensitive(false);
        area_grab_button.set_tooltip_text(Some(&get_bundle("wayland-tooltip", None)));
    }
    // Disable follow mouse option
    #[cfg(target_os = "windows")]
    {
        follow_mouse_switch.set_active(false);
        follow_mouse_switch.set_sensitive(false);
    }

    // Entries
    filename_entry.set_placeholder_text(Some(&get_bundle("file-name", None)));
    command_entry.set_placeholder_text(Some(&get_bundle("default-command", None)));
    filename_entry.set_text(&config_management::get("default", "filename"));
    command_entry.set_text(&config_management::get("default", "command"));

    // Format combobox
    format_chooser_combobox.append(Some("mp4"), &get_bundle("mp4-format", None));
    format_chooser_combobox.append(
        Some("mkv"),
        &get_bundle("mkv-format", None),
    );
    format_chooser_combobox.append(Some("webm"), &get_bundle("webm-format", None));
    format_chooser_combobox.append(Some("gif"), &get_bundle("gif-format", None));
    format_chooser_combobox.append(Some("avi"), &get_bundle("avi-format", None));
    format_chooser_combobox.append(Some("wmv"), &get_bundle("wmv-format", None));
    format_chooser_combobox.append(Some("nut"), &get_bundle("nut-format", None));
    format_chooser_combobox.set_active(Some(config_management::get("default", "format").parse::<u32>().unwrap_or(0u32)));

    // Get audio sources
    let input_device = host_audio_device.input_devices()?;
    let sources_descriptions: Vec<String> = input_device
        .filter_map(|device| device.name().ok())
        .collect();
    let host_output_device = host_audio_device.default_output_device();
    let output_device = if host_output_device.is_some() {
        host_output_device.unwrap().name()?
    } else {
      String::new()
    };

    audio_source_combobox.append(Some("default"), &get_bundle("audio-input", None));
    for (id, audio_source) in sources_descriptions.iter().enumerate() {
        audio_source_combobox.append(Some(id.to_string().as_str()), audio_source);
    }
    audio_source_combobox.set_active(Some(0));

    // Switchs
    audio_input_switch.set_active(config_management::get_bool("default", "audio_input_check"));
    follow_mouse_switch.set_active(config_management::get_bool("default", "followmousecheck"));
    hide_switch.set_active(config_management::get_bool("default", "hidecheck"));
    mouse_switch.set_active(config_management::get_bool("default", "mousecheck"));
    audio_output_switch.set_active(config_management::get_bool("default", "speakercheck"));
    tray_switch.set_active(config_management::get_bool("default", "traycheck"));
    video_switch.set_active(config_management::get_bool("default", "videocheck"));
    area_switch.set_label(Some(&get_bundle("show-area", None)));
    audio_input_switch.set_label(Some(&get_bundle("record-audio", None)));
    follow_mouse_switch.set_label(Some(&get_bundle("follow-mouse", None)));
    hide_switch.set_label(Some(&get_bundle("auto-hide", None)));
    mouse_switch.set_label(Some(&get_bundle("show-mouse", None)));
    audio_output_switch.set_label(Some(&get_bundle("record-speaker", None)));
    tray_switch.set_label(Some(&get_bundle("tray-minimize", None)));
    video_switch.set_label(Some(&get_bundle("record-video", None)));
    area_switch.set_tooltip_text(Some(&get_bundle("show-area-tooltip", None)));
    audio_input_switch.set_tooltip_text(Some(&get_bundle("audio-input-tooltip", None)));
    #[cfg(target_os = "windows")]
    follow_mouse_switch.set_tooltip_text(Some(&get_bundle("windows-unsupported-tooltip", None)));
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    follow_mouse_switch.set_tooltip_text(Some(&get_bundle("follow-mouse-tooltip", None)));
    hide_switch.set_tooltip_text(Some(&get_bundle("hide-tooltip", None)));
    mouse_switch.set_tooltip_text(Some(&get_bundle("mouse-tooltip", None)));
    audio_output_switch.set_tooltip_text(Some(&get_bundle("speaker-tooltip", None)));
    tray_switch.set_tooltip_text(Some(&get_bundle("tray-minimize-tooltip", None)));
    video_switch.set_tooltip_text(Some(&get_bundle("video-tooltip", None)));

    let _mouse_switch = mouse_switch.clone();
    let _video_switch = video_switch.clone();
    audio_input_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "audio_input_check", switch.is_active());
        if !switch.is_active() && !_video_switch.is_active() {
            _mouse_switch.set_sensitive(true);
        }
    });
    follow_mouse_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "followmousecheck", switch.is_active());
    });
    hide_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "hidecheck", switch.is_active());
    });
    mouse_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "mousecheck", switch.is_active());
    });
    audio_output_switch.connect_toggled(|switch: &CheckButton| {
        config_management::set_bool("default", "speakercheck", switch.is_active());
    });
    let _audio_input_switch = audio_input_switch.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    let _mouse_switch = mouse_switch.clone();
    video_switch.connect_toggled(move |switch: &CheckButton| {
        config_management::set_bool("default", "videocheck", switch.is_active());
        if switch.is_active() {
            _follow_mouse_switch.set_sensitive(true);
            _mouse_switch.set_sensitive(true);
        } else {
            _mouse_switch.set_active(false);
            _mouse_switch.set_sensitive(false);
            _follow_mouse_switch.set_active(false);
            _follow_mouse_switch.set_sensitive(false);
        }
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
    audio_bitrate_spin.set_tooltip_text(Some(&get_bundle("audio-bitrate-tooltip", None)));
    delay_spin.set_tooltip_text(Some(&get_bundle("delay-tooltip", None)));
    frames_spin.set_tooltip_text(Some(&get_bundle("frames-tooltip", None)));
    video_bitrate_spin.set_tooltip_text(Some(&get_bundle("video-bitrate-tooltip", None)));
    frames_spin.set_value(
        config_management::get("default",
                               &format!
                               ("frame-{}",
                                &format_chooser_combobox.active().unwrap().to_string()))
            .parse::<f64>()
            .unwrap_or(0f64),
    );
    audio_bitrate_spin.set_value(
        config_management::get("default", "audiobitrate")
            .parse::<f64>()
            .unwrap_or(0f64),
    );
    delay_spin.set_value(
        config_management::get("default", "delay")
            .parse::<f64>()
            .unwrap_or(0f64),
    );
    video_bitrate_spin.set_value(
        config_management::get("default",
                               &format!
                               ("videobitrate-{}",
                                &format_chooser_combobox.active().unwrap().to_string()))
            .parse::<f64>()
            .unwrap_or(0f64),
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
                    .unwrap_or(0f64),
            );
            _video_bitrate_spin.set_value(
                config_management::get("default",
                                       &format!
                                       ("videobitrate-{}",
                                        &format_chooser_combobox.active().unwrap().to_string()))
                    .parse::<f64>()
                    .unwrap_or(0f64),
            );
        }
    });

    let _audio_bitrate_spin = audio_bitrate_spin.to_owned();
    audio_bitrate_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               "audio_bitrate",
                               _audio_bitrate_spin.value().to_string().as_str());
    });
    let _delay_spin = delay_spin.to_owned();
    delay_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               "delay",
                               _delay_spin.value().to_string().as_str());
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
    let _format_chooser_combobox = format_chooser_combobox.clone();
    let _video_bitrate_spin = video_bitrate_spin.to_owned();
    video_bitrate_spin.connect_value_changed(move |_| {
        config_management::set("default",
                               &format!
                               ("videobitrate-{}",
                                &_format_chooser_combobox.active().unwrap().to_string()),
                               _video_bitrate_spin.value().to_string().as_str());
     });

    // Labels
    audio_bitrate_label.set_label(&get_bundle("audio-bitrate", None));
    audio_source_label.set_label(&get_bundle("audio-source", None));
    delay_label.set_label(&get_bundle("delay", None));
    command_label.set_label(&get_bundle("run-command", None));
    frames_label.set_label(&get_bundle("frames", None));
    video_bitrate_label.set_label(&get_bundle("video-bitrate", None));

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
        .set_file(&File::for_path(&config_management::get(
            "default", "folder",
        )))
        .unwrap();
    let folder_chooser = Some(File::for_path(&config_management::get(
        "default", "folder",
    )))
    .unwrap();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let folder_chooser_name = folder_chooser.basename().unwrap().to_string_lossy().to_string();
    folder_chooser_label.set_label(&folder_chooser_name);
    let folder_chooser_icon = config_management::folder_icon(Some(folder_chooser_name.as_str()));
    folder_chooser_image.set_icon_name(Some(folder_chooser_icon));
    folder_chooser_button.set_tooltip_text(Some(&get_bundle("folder-tooltip", None)));
    // Show file chooser dialog
    folder_chooser_button.connect_clicked(glib::clone!(@strong folder_chooser_native => move |_| {
        let error_dialog = _error_dialog.clone();
        let error_message = _error_message.clone();
        folder_chooser_native.connect_response
                             (glib::clone!(@strong folder_chooser_native, @strong folder_chooser_label,
                                           @strong folder_chooser_image => move |_, response| {
                                               let text_buffer = TextBuffer::new(None);
                                               if response == adw::gtk::ResponseType::Accept {
                                                   if folder_chooser_native.file().is_none() {
                                                       text_buffer.set_text("Failed to get save file path.");
                                                       error_message.set_buffer(Some(&text_buffer));
                                                       error_dialog.show();
                                                   }
                                                   let folder_chooser = folder_chooser_native.file().unwrap_or_else
                                                       (||
                                                        File::for_path(&config_management::get(
                                                            "default", "folder",
                                                        ))); // Default

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
    about_button.set_tooltip_text(Some(&get_bundle("about-tooltip", None)));
    let _about_dialog: AboutDialog = about_dialog.to_owned();
    about_button.set_label(&get_bundle("about", None));
    about_button.connect_clicked(move |_| {
        _about_dialog.show();
        _about_dialog.set_hide_on_close(true);
    });

    // Buttons
    let area_capture: Rc<RefCell<area_capture::AreaCapture>> =
        Rc::new(RefCell::new(area_capture::AreaCapture::new()?));
    #[cfg(target_os = "windows")]
    let window_title: Rc<RefCell<area_capture::Title>> =
        Rc::new(RefCell::new(area_capture::Title::new()?));

    area_grab_label.set_label(&get_bundle("select-area", None));
    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    let _area_switch = area_switch.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    area_grab_button.connect_clicked(move |_| {
        config_management::set("default", "mode", "area");
        _area_chooser_window.show();
        if area_capture::show_size(
            _area_chooser_window.clone(),
            area_size_bottom_label.clone(),
            area_size_top_label.clone(),
        ).is_err() {
            let text_buffer = TextBuffer::new(None);
            text_buffer.set_text("Failed to get area size value.");
            _error_message.set_buffer(Some(&text_buffer));
            _error_dialog.show();
        }
        _area_switch.set_active(config_management::get_bool("default", "areacheck"));
        _area_switch.set_sensitive(true);
    });

    area_apply_label.set_label(&get_bundle("apply", None));
    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    area_set_button.connect_clicked(move |_| {
        let text_buffer = TextBuffer::new(None);
        #[cfg(target_os = "windows")]
        if _area_capture
            .borrow_mut()
            .get_active_window().is_err() {
                text_buffer.set_text("Failed to get area size value.");
                _error_message.set_buffer(Some(&text_buffer));
                _error_dialog.show();
            }
        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        if _area_capture
            .borrow_mut()
            .get_window_by_name(_area_chooser_window.title().unwrap().as_str()).is_err() {
                text_buffer.set_text("Failed to get area size value.");
                _error_message.set_buffer(Some(&text_buffer));
                _error_dialog.show();
            }
        _area_chooser_window.hide();
    });

    let _area_chooser_window = area_chooser_window.clone();
    let mut _area_capture = area_capture.clone();
    let _area_switch = area_switch.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let record_window: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let window_grab_button_record_window: Rc<RefCell<bool>> = record_window.clone();
    let screen_grab_button_record_window: Rc<RefCell<bool>> = record_window.clone();
    screen_grab_button.set_tooltip_text(Some(&get_bundle("screen-tooltip", None)));
    screen_grab_label.set_label(&get_bundle("select-screen", None));
    screen_grab_button.connect_clicked(move |_| {
        let text_buffer = TextBuffer::new(None);
        config_management::set_bool("default", "areacheck", _area_switch.is_active());
        _area_switch.set_active(false);
        _area_switch.set_sensitive(false);
        config_management::set("default", "mode", "screen");
        screen_grab_button_record_window.replace(false);
        _area_chooser_window.hide();
        if _area_capture.borrow_mut().reset().is_err() {
            text_buffer.set_text("Failed to reset area_capture value.");
            _error_message.set_buffer(Some(&text_buffer));
            _error_dialog.show();
        }
    });

    let _area_chooser_window: Window = area_chooser_window.clone();
    let mut _area_capture: Rc<RefCell<area_capture::AreaCapture>> = area_capture.clone();
    let _area_switch = area_switch.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    window_grab_button.set_tooltip_text(Some(&get_bundle("window-tooltip", None)));
    window_grab_label.set_label(&get_bundle("select-window", None));
    #[cfg(target_os = "windows")]
    let mut _window_title: Rc<RefCell<area_capture::Title>> = window_title.clone();
    window_grab_button.connect_clicked(move |_| {
        let text_buffer = TextBuffer::new(None);
        config_management::set_bool("default", "areacheck", _area_switch.is_active());
        _area_switch.set_active(false);
        _area_switch.set_sensitive(false);
        config_management::set("default", "mode", "window");
        _area_chooser_window.hide();
        if is_wayland() {
            window_grab_button_record_window.replace(true);
        } else {
            #[cfg(target_os = "windows")]
            {
                select_window_label.set_label(&get_bundle("click-window", None));
                select_window.show();

                let area_capture = _area_capture.clone();
                let error_message = _error_message.clone();
                let error_dialog = error_dialog.clone();
                let _select_window = select_window.clone();
                let window_title = _window_title.clone();
                glib::timeout_add_local(1000, move || {
                    let clicked = area_capture::check_input();
                    if clicked {
                        _select_window.hide();
                        if window_title.borrow_mut().get_title().is_err() {
                            text_buffer.set_text("Failed to get window title.");
                            error_message.set_buffer(Some(&text_buffer));
                            error_dialog.show();
                        }
                        return glib::source::Continue(false);
                    } else if !clicked {
                        _select_window.hide();
                        return glib::source::Continue(false);
                    }
                    glib::source::Continue(true)
                });
            }

            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            {
                if _area_capture.borrow_mut().get_area().is_err() {
                    text_buffer.set_text("Failed to get window info.");
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                }
            }}
    });

    // Record struct values
    let audio_output_id = if audio_output_switch.is_active() {
        output_device
    } else {
        String::new()
    };
    let mode = if area_grab_button.is_active() {
        RecordMode::Area
    } else if window_grab_button.is_active() {
        RecordMode::Window
    } else {
        RecordMode::Screen
    };
    #[cfg(target_os = "windows")]
    let window_title = window_title.borrow_mut().title.clone();

    // Init record struct
    #[cfg(target_os = "windows")]
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        audio_input_id: audio_source_combobox.clone(),
        audio_output_id,
        filename: (
            folder_chooser_native,
            filename_entry,
            format_chooser_combobox,
        ),
        output: String::new(),
        temp_input_audio_filename: String::new(),
        temp_output_audio_filename: String::new(),
        temp_video_filename: String::new(),
        window_title,
        saved_filename: String::new(),
        height: None,
        input_audio_process: None,
        output_audio_process: None,
        video_process: None,
        audio_record_bitrate: audio_bitrate_spin,
        record_delay: delay_spin,
        record_frames: frames_spin,
        video_record_bitrate: video_bitrate_spin,
        follow_mouse: follow_mouse_switch,
        record_mouse: mouse_switch,
        show_area: area_switch
    }));
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        audio_input_id: audio_source_combobox.clone(),
        audio_output_id,
        filename: (
            folder_chooser_native,
            filename_entry,
            format_chooser_combobox,
        ),
        output: String::new(),
        temp_input_audio_filename: String::new(),
        temp_output_audio_filename: String::new(),
        temp_video_filename: String::new(),
        saved_filename: String::new(),
        height: None,
        input_audio_process: None,
        output_audio_process: None,
        video_process: None,
        audio_record_bitrate: audio_bitrate_spin,
        record_delay: delay_spin.clone(),
        record_frames: frames_spin,
        video_record_bitrate: video_bitrate_spin,
        follow_mouse: follow_mouse_switch,
        record_mouse: mouse_switch,
        show_area: area_switch
    }));

    // Record button
    delay_window_button.set_label(&get_bundle("delay-window-stop", None));
    delay_window_title.set_label(&get_bundle("delay-title", None));
    record_button.set_tooltip_text(Some(&get_bundle("record-tooltip", None)));
    record_label.set_label(&get_bundle("record", None));
    let _audio_input_switch = audio_input_switch.clone();
    let _audio_output_switch = audio_output_switch.clone();
    //let bundle_msg = get_bundle("already-exist", None);
    let _delay_spin = delay_spin.clone();
    let _delay_window = delay_window.clone();
    let _delay_window_button = delay_window_button.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    //let main_context = glib::MainContext::default();
    let _main_window = main_window.clone();
    let _play_button = play_button.clone();
    let _record_button = record_button.clone();
    let _record_time_label = record_time_label.clone();
    let _stop_button = stop_button.clone();
    let _video_switch = video_switch.clone();
    //let wayland_record = main_context.block_on(WaylandRecorder::new());
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    record_button.connect_clicked(move |_| {
        match _ffmpeg_record_interface.borrow_mut().get_filename() {
            Err(error) => {
                let text_buffer = TextBuffer::new(None);
                text_buffer.set_text(&format!("{}", error));
                _error_message.set_buffer(Some(&text_buffer));
                _error_dialog.show();
            },
            Ok(_) => {
                if !_audio_input_switch.is_active() &&
                    !_audio_output_switch.is_active() &&
                    !_video_switch.is_active()
                {
                    // Do nothing
                } else {
                    _delay_window_button.set_active(false);
                    if _delay_spin.value() as u16 > 0 {
                        recording_delay(
                            _delay_spin.clone(),
                            _delay_spin.value() as u16,
                            delay_window.clone(),
                            _delay_window_button.clone(),
                            delay_window_label.clone(),
                            _record_button.clone(),
                        );
                    } else if _delay_spin.value() as u16 == 0 {
                        let _area_capture = area_capture.borrow_mut();
                        _audio_input_switch.set_sensitive(false);
                        _audio_output_switch.set_sensitive(false);
                        _video_switch.set_sensitive(false);
                        start_timer(record_time_label.clone());
                        record_time_label.set_visible(true);
                        if hide_switch.is_active() {
                            _main_window.minimize();
                        }
                        _play_button.hide();
                        _record_button.hide();
                        _stop_button.show();
                        if _audio_input_switch.is_active() {
                            match _ffmpeg_record_interface.borrow_mut().start_input_audio() {
                                Ok(_) => {
                                    // Do nothing
                                },
                                Err(error) => {
                                    _audio_input_switch.set_sensitive(true);
                                    _audio_output_switch.set_sensitive(true);
                                    _video_switch.set_sensitive(true);
                                    _record_button.show();
                                    _stop_button.hide();
                                    let text_buffer = TextBuffer::new(None);
                                    text_buffer.set_text(&format!("{}", error));
                                    _error_message.set_buffer(Some(&text_buffer));
                                    _error_dialog.show();
                                },
                            }
                        }
                        if _audio_output_switch.is_active() {
                            match _ffmpeg_record_interface.borrow_mut().start_output_audio() {
                                Ok(_) => {
                                    // Do nothing
                                },
                                Err(error) => {
                                    _audio_input_switch.set_sensitive(true);
                                    _audio_output_switch.set_sensitive(true);
                                    _video_switch.set_sensitive(true);
                                    _record_button.show();
                                    _stop_button.hide();
                                    let text_buffer = TextBuffer::new(None);
                                    text_buffer.set_text(&format!("{}", error));
                                    _error_message.set_buffer(Some(&text_buffer));
                                    _error_dialog.show();
                                },
                            }
                        }
                        if _video_switch.is_active() {
                            match _ffmpeg_record_interface.borrow_mut().start_video(
                                _area_capture.x,
                                _area_capture.y,
                                _area_capture.width,
                                _area_capture.height,
                                mode
                            ) {
                                Ok(_) => {
                                    // Do nothing
                                },
                                Err(error) => {
                                    _audio_input_switch.set_sensitive(true);
                                    _audio_output_switch.set_sensitive(true);
                                    _video_switch.set_sensitive(true);
                                    _record_button.show();
                                    _stop_button.hide();
                                    let text_buffer = TextBuffer::new(None);
                                    text_buffer.set_text(&format!("{}", error));
                                    _error_message.set_buffer(Some(&text_buffer));
                                    _error_dialog.show();
                                },
                            }
                        }
                    }
                }
            },
        }
    });

    // Stop record button
    stop_button.set_tooltip_text(Some(&get_bundle("stop-tooltip", None)));
    stop_label.set_label(&get_bundle("stop-recording", None));
    let _audio_input_switch = audio_input_switch.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let _play_button = play_button.clone();
    let _audio_output_switch = audio_output_switch.clone();
    let _stop_button = stop_button.clone();
    let _video_switch = video_switch.clone();
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    stop_button.connect_clicked(move |_| {
        _record_time_label.set_visible(false);
        stop_timer(_record_time_label.clone());
        if _audio_input_switch.is_active() {
            match _ffmpeg_record_interface.borrow_mut().stop_input_audio() {
                Ok(_) => {
                    // Do nothing
                    },
                Err(error) => {
                    _audio_input_switch.set_sensitive(true);
                    _audio_output_switch.set_sensitive(true);
                    _video_switch.set_sensitive(true);
                    record_button.show();
                    _stop_button.hide();
                    let text_buffer = TextBuffer::new(None);
                    text_buffer.set_text(&format!("{}", error));
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                },
            }
        }
        if _audio_output_switch.is_active() {
            match _ffmpeg_record_interface.borrow_mut().stop_output_audio() {
                Ok(_) => {
                    // Do nothing
                    },
                Err(error) => {
                    _audio_input_switch.set_sensitive(true);
                    _audio_output_switch.set_sensitive(true);
                    _video_switch.set_sensitive(true);
                    record_button.show();
                    _stop_button.hide();
                    let text_buffer = TextBuffer::new(None);
                    text_buffer.set_text(&format!("{}", error));
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                },
            }
        }
        if _video_switch.is_active() {
            match _ffmpeg_record_interface.borrow_mut().stop_video() {
                Ok(_) => {
                    // Do nothing
                },
                Err(error) => {
                    _audio_input_switch.set_sensitive(true);
                    _audio_output_switch.set_sensitive(true);
                    _video_switch.set_sensitive(true);
                    record_button.show();
                    _stop_button.hide();
                    let text_buffer = TextBuffer::new(None);
                    text_buffer.set_text(&format!("{}", error));
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                },
            }
        }
        _audio_input_switch.set_sensitive(true);
        _audio_output_switch.set_sensitive(true);
        _video_switch.set_sensitive(true);
        record_button.show();
        _stop_button.hide();
        let file_name = _ffmpeg_record_interface.borrow_mut().saved_filename.clone();
        if Path::new(&file_name).try_exists().is_ok() {
            _play_button.show();
            _play_button.set_tooltip_text(Some(&get_bundle("play-tooltip", None)));
        } else  {
            _play_button.show();
            _play_button.set_sensitive(false);
            _play_button.set_tooltip_text(Some(&get_bundle("play-inactive-tooltip", None)));
        }
    });

    // Delay window button
    let _delay_window_button = delay_window_button.clone();
    delay_window_button.connect_clicked(move |_| {});

    // Play button
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    play_button.connect_clicked(move |_| {
        let file_name = _ffmpeg_record_interface.borrow_mut().saved_filename.clone();
        match play_record(&file_name) {
            Ok(_) => {
                // Do nothing
            },
            Err(error) => {
                let text_buffer = TextBuffer::new(None);
                text_buffer.set_text(&format!("{}", error));
                _error_message.set_buffer(Some(&text_buffer));
                _error_dialog.show();
            },
        }
    });

    // About dialog
    let mut about_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/blue-recorder.svg"));

    if !about_icon_path.exists() {
        about_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
                .add("blue-recorder.svg"),
        ))
        .unwrap();
    }

    about_dialog.set_comments(Some(&get_bundle("dialog-comment", None)));
    about_dialog.set_copyright(Some(&get_bundle("copy-right", None)));
    about_dialog.set_license(Some(&get_bundle("license", None)));
    let logo = Image::from_file(&about_icon_path.to_str().unwrap());
    about_dialog.set_logo(logo.paintable().as_ref());
    about_dialog.set_modal(true);
    about_dialog.set_program_name(Some(&get_bundle("blue-recorder", None)));
    about_dialog.set_transient_for(Some(&main_window));
    about_dialog.set_version(Some("0.3.0"));
    about_dialog.set_website(Some("https://github.com/xlmnxp/blue-recorder/"));
    about_dialog.set_website_label(&get_bundle("website", None));
    about_dialog.set_wrap_license(true);
    // Authors
    about_dialog.add_credit_section(
        &get_bundle("authors", None),
        &[&get_bundle("address-abdullah-al-baroty", None),
          &get_bundle("address-alessandro-toia", None),
          &get_bundle("address-chibani", None),
          &get_bundle("address-hamir-mahal", None),
          &get_bundle("address-hanny-sabbagh", None),
          &get_bundle("address-salem-yaslem", None),
          &get_bundle("address-suliman-altassan", None),
        ]
    );
    // Patreon suppoters
    about_dialog.add_credit_section(
        &get_bundle("patreon", None),
        &[&get_bundle("address-ahmad-gharib", None),
          &get_bundle("address-medium", None),
          &get_bundle("address-william-grunow", None),
          &get_bundle("address-alex-benishek", None),
        ]
    );
    // Designers
    about_dialog.add_credit_section(
        &get_bundle("design", None),
        &[&get_bundle("address-abdullah-al-baroty", None),
          &get_bundle("address-mustapha-assabar", None),
        ]
    );
    // Translators
    about_dialog.add_credit_section(
        &get_bundle("translate", None),
        &[&get_bundle("address-ake-engelbrektson", None),
          &get_bundle("address-amerey", None),
          &get_bundle("address-gmou3", None),
          &get_bundle("address-larry-wei", None),
          &get_bundle("address-mark-wagie", None),
          &get_bundle("address-albanobattistella", None),
          &get_bundle("address-mr-Narsus", None),
       ]
    );

    // Windows
    // Hide area chooser after it deleted.
    let _area_chooser_window = area_chooser_window.clone();
    area_chooser_window.connect_close_request(move |_| {
        _area_chooser_window.hide();
        adw::gtk::Inhibit(true)
    });

    // Stop recording before close the application
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    main_window.connect_close_request(move |main_window| {
        _ffmpeg_record_interface.borrow_mut().kill().unwrap();
        main_window.destroy();
        adw::gtk::Inhibit(true)
    });

    // Apply CSS
    let display = adw::gdk::Display::default().unwrap();
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("styles/global.css").as_bytes());
    adw::gtk::StyleContext::add_provider_for_display(
        &display,
        &provider,
        adw::gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    main_window.show();

    Ok(())
}
