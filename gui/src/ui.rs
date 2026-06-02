use adw::{Application, Window};
use adw::gio::File;
use adw::gtk::{AboutDialog, Box as GtkBox, Builder, Button, CheckButton, ComboBoxText, CssProvider, Entry, Expander, FileChooserNative,
               FileChooserAction, Image, Label, MessageDialog, SpinButton, TextBuffer, TextView, ToggleButton, Widget};
use adw::prelude::*;
use anyhow::Result;
#[cfg(any(target_os = "freebsd", target_os = "linux"))]
use blue_recorder_core::ffmpeg_linux::Ffmpeg;
#[cfg(target_os = "windows")]
use blue_recorder_core::ffmpeg_windows::Ffmpeg;
use blue_recorder_core::utils::{is_wayland, play_record, RecordMode};
#[cfg(any(target_os = "freebsd", target_os = "linux"))]
use blue_recorder_core::wayland_linux::WaylandRecorder;
#[cfg(target_os = "windows")]
use cpal::traits::{DeviceTrait, HostTrait};
use std::cell::RefCell;
use std::ops::Add;
use std::path::Path;
use std::rc::Rc;

use crate::{area_capture, config_management, fluent::get_bundle};
use crate::timer::{RecordClick, recording_delay, start_timer, stop_timer};
use crate::utils::{audio_output_source, build_filename, disable_input_widgets,
                   enable_input_widgets, is_overwrite, sources_descriptions_list};

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
    #[cfg(target_os = "windows")]
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
    let audio_input_switch: CheckButton = builder.object("audio_input_switch").unwrap();
    let audio_output_switch: CheckButton = builder.object("speakerswitch").unwrap();
    let audio_source_combobox: ComboBoxText = builder.object("audiosource").unwrap();
    let audio_source_label: Label = builder.object("audio_source_label").unwrap();
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
    let app_title: Label = builder.object("app_title").unwrap();
    let processing_box: GtkBox = builder.object("processing_box").unwrap();
    let processing_label: Label = builder.object("processing_label").unwrap();
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
    area_chooser_window.set_transient_for(Some(&main_window));
    area_chooser_window.set_title(Some(&get_bundle("area-chooser", None))); // Title is hidden
    error_dialog.set_transient_for(Some(&main_window));
    select_window.set_transient_for(Some(&main_window));
    main_window.set_application(Some(application));
    main_window.set_title(Some(&get_bundle("blue-recorder", None))); // used by taskbar
    app_title.set_label(&get_bundle("blue-recorder", None));

    // Play button hidden until a recording exists; stop button always visible.
    play_button.hide();
    stop_button.hide();

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
    #[cfg(target_os = "windows")]
    let input_device = host_audio_device.input_devices()?;
    #[cfg(target_os = "windows")]
    let sources_descriptions: Vec<String> = input_device
        .filter_map(|device| device.name().ok())
        .collect();
    #[cfg(target_os = "windows")]
    let host_output_device = host_audio_device.default_output_device();
    #[cfg(target_os = "windows")]
    let audio_output_source = if host_output_device.is_some() {
        host_output_device.unwrap().name()?
    } else {
        String::new()
    };

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let sources_descriptions: Vec<String> = match sources_descriptions_list() {
        Ok(descriptions) => descriptions,
        Err(_) => {
            Vec::new()
        }
    };
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let audio_output_source = match audio_output_source() {
        Ok(audio_output_source) => {
            audio_output_source
        }
        Err(_) => {
            String::new()
        }
    };

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
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
    audio_input_switch.set_tooltip_text(Some(&get_bundle("mic-tooltip", None)));
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
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            _follow_mouse_switch.set_sensitive(true);
            _mouse_switch.set_sensitive(true);
        } else {
            _mouse_switch.set_active(false);
            _mouse_switch.set_sensitive(false);
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            _follow_mouse_switch.set_active(false);
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            _follow_mouse_switch.set_sensitive(false);
        }
    });

    // Resolve an icon file from the data directory, falling back to DATA_DIR env.
    let resolve_icon = |name: &str| -> std::path::PathBuf {
        let mut path = {
            let mut dir = std::env::current_exe().unwrap();
            dir.pop();
            dir
        }
        .join(Path::new("data").join(name));
        if !path.exists() {
            path = std::fs::canonicalize(Path::new(
                &std::env::var("DATA_DIR")
                    .unwrap_or_else(|_| String::from("data/"))
                    .add(name),
            ))
            .unwrap_or_else(|_| path);
        }
        path
    };

    // Apply the correct icon set for the current dark/light state and
    // re-run automatically whenever the user switches colour scheme.
    let apply_icons = {
        let area_icon   = area_grab_icon.clone();
        let screen_icon = screen_grab_icon.clone();
        let window_icon = window_grab_icon.clone();
        let resolve     = resolve_icon.clone();
        move |dark: bool| {
            let suffix = if dark { "-white" } else { "" };
            area_icon.set_from_file(Some(resolve(&format!("screenshot-ui-area-symbolic{}.svg",    suffix))));
            screen_icon.set_from_file(Some(resolve(&format!("screenshot-ui-display-symbolic{}.svg", suffix))));
            window_icon.set_from_file(Some(resolve(&format!("screenshot-ui-window-symbolic{}.svg",  suffix))));
        }
    };

    // A small dynamic provider that sets grab-button label colour based on
    // the current colour scheme. Loaded once here and reloaded on each change.
    let grab_color_provider = CssProvider::new();
    let update_grab_colors = {
        let p = grab_color_provider.clone();
        move |dark: bool| {
            let color = if dark { "#ffffff" } else { "#1a1a1a" };
            p.load_from_data(
                format!(
                    "#area_grab_button:checked label, \
                     #screen_grab_button:checked label, \
                     #window_grab_button:checked label {{ color: {}; }}",
                    color
                )
                .as_bytes(),
            );
        }
    };

    let style_manager = adw::StyleManager::default();
    update_grab_colors(style_manager.is_dark());
    apply_icons(style_manager.is_dark());
    style_manager.connect_dark_notify(move |m| {
        let dark = m.is_dark();
        update_grab_colors(dark);
        apply_icons(dark);
    });

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
    {
        let fcn = folder_chooser_native.clone();
        let fcl = folder_chooser_label.clone();
        let fci = folder_chooser_image.clone();
        let ed = _error_dialog.clone();
        let em = _error_message.clone();
        folder_chooser_button.connect_clicked(move |_| {
            let fcn_inner = fcn.clone();
            let fcl_inner = fcl.clone();
            let fci_inner = fci.clone();
            let ed_inner = ed.clone();
            let em_inner = em.clone();
            fcn.connect_response(move |_, response| {
                let text_buffer = TextBuffer::new(None);
                if response == adw::gtk::ResponseType::Accept {
                    if fcn_inner.file().is_none() {
                        text_buffer.set_text("Failed to get save file path.");
                        em_inner.set_buffer(Some(&text_buffer));
                        ed_inner.show();
                    }
                    let folder_chooser = fcn_inner.file().unwrap_or_else(|| {
                        File::for_path(&config_management::get("default", "folder"))
                    });
                    let folder_chooser_name = folder_chooser.basename().unwrap();
                    fcl_inner.set_label(&folder_chooser_name.to_string_lossy());
                    let folder_chooser_icon = config_management::folder_icon(folder_chooser_name.to_str());
                    fci_inner.set_icon_name(Some(folder_chooser_icon));
                }
                fcn_inner.hide();
            });
            fcn.show();
        });
    }

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
    let _window_title = window_title.clone();
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

                let error_message = _error_message.clone();
                let error_dialog = _error_dialog.clone();
                let _select_window = select_window.clone();
                let window_title = _window_title.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
                    let clicked = area_capture::check_input();
                    if clicked {
                        _select_window.hide();
                        if window_title.borrow_mut().get_title().is_err() {
                            text_buffer.set_text("Failed to get window title.");
                            error_message.set_buffer(Some(&text_buffer));
                            error_dialog.show();
                        }
                        return glib::ControlFlow::Break;
                    } else if !clicked {
                        _select_window.hide();
                        return glib::ControlFlow::Break;
                    }
                    glib::ControlFlow::Continue
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

    // Disable mouse cursor capture if video record is not active
    if !video_switch.is_active() {
        mouse_switch.set_sensitive(false);
        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        follow_mouse_switch.set_sensitive(false);
    }

    // Input widgets list
    let mut input_widgets: Vec<Widget> = vec![
        filename_entry.clone().into(),
        folder_chooser_button.clone().into(),
        format_chooser_combobox.clone().into(),
        screen_grab_button.clone().into(),
        window_grab_button.clone().into(),
        video_switch.clone().into(),
        frames_label.clone().into(),
        frames_spin.clone().into(),
        delay_label.clone().into(),
        delay_spin.clone().into(),
        hide_switch.clone().into(),
        video_bitrate_label.clone().into(),
        video_bitrate_spin.clone().into(),
        audio_bitrate_label.clone().into(),
        audio_bitrate_spin.clone().into(),
        audio_source_label.clone().into(),
        audio_source_combobox.clone().into(),
        command_label.clone().into(),
        command_entry.clone().into()
    ];

    // Temporary solution
    if !is_wayland() {
        // Keep area_selection disaled in wayland
        input_widgets.push(area_grab_button.clone().into());
    }

    // Disable show area check button
    if !area_grab_button.is_active() {
        area_switch.set_active(false);
        area_switch.set_sensitive(false);
    }

    // Disable audio input record
    if sources_descriptions.is_empty() {
        audio_input_switch.set_active(false);
        audio_input_switch.set_sensitive(false);
        audio_input_switch.set_tooltip_text(Some(&get_bundle("no-audio-input-tooltip", None)));
    } else {
        input_widgets.push(audio_input_switch.clone().into());
    }

    // Disable audio output record
    if audio_output_source.is_empty() {
        audio_output_switch.set_active(false);
        audio_output_switch.set_sensitive(false);
        audio_output_switch.set_tooltip_text(Some(&get_bundle("no-audio-output-tooltip", None)));
    } else {
        input_widgets.push(audio_output_switch.clone().into());
    }

    // Init record struct — all fields are plain data; widgets are read just
    // before recording starts via apply_recording_config() below.
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        audio_input_id: String::new(),
        audio_output_id: audio_output_source,
        filename: String::new(),
        output: String::new(),
        audio_record_bitrate: 0,
        record_delay: 0,
        record_frames: 0,
        video_record_bitrate: 0,
        audio_input_enabled: false,
        audio_output_enabled: false,
        follow_mouse: false,
        record_mouse: false,
        show_area: false,
        video_enabled: false,
        saved_filename: String::new(),
        temp_video_filename: String::new(),
        temp_input_audio_filename: String::new(),
        temp_output_audio_filename: String::new(),
        width: None,
        height: None,
        input_audio_process: None,
        output_audio_process: None,
        video_process: None,
        wayland_recorder: async_std::task::block_on(WaylandRecorder::new()),
    }));

    #[cfg(target_os = "windows")]
    let ffmpeg_record_interface: Rc<RefCell<Ffmpeg>> = Rc::new(RefCell::new(Ffmpeg {
        audio_input_id: String::new(),
        audio_output_id: audio_output_source,
        filename: String::new(),
        output: String::new(),
        audio_record_bitrate: 0,
        record_delay: 0,
        record_frames: 0,
        video_record_bitrate: 0,
        audio_input_enabled: false,
        audio_output_enabled: false,
        follow_mouse: false,
        record_mouse: false,
        show_area: false,
        video_enabled: false,
        saved_filename: String::new(),
        temp_video_filename: String::new(),
        temp_input_audio_filename: String::new(),
        temp_output_audio_filename: String::new(),
        width: None,
        height: None,
        input_audio_process: None,
        output_audio_process: None,
        video_process: None,
    }));

    // Record button
    delay_window_button.set_label(&get_bundle("delay-window-stop", None));
    delay_window_title.set_label(&get_bundle("delay-title", None));
    record_button.set_tooltip_text(Some(&get_bundle("record-tooltip", None)));
    record_label.set_label(&get_bundle("record", None));
    let _area_grab_button = area_grab_button.clone();
    let _area_switch = area_switch.clone();
    let _audio_input_switch = audio_input_switch.clone();
    let _audio_output_switch = audio_output_switch.clone();
    let _delay_spin = delay_spin.clone();
    let _delay_window = delay_window.clone();
    let _delay_window_button = delay_window_button.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    let mut _input_widgets: Vec<Widget> = input_widgets.clone();
    let _main_window = main_window.clone();
    let _mouse_switch = mouse_switch.clone();
    let _play_button = play_button.clone();
    let _record_button = record_button.clone();
    let _record_time_label = record_time_label.clone();
    let _stop_button = stop_button.clone();
    let _video_switch = video_switch.clone();
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    // Widgets read at record-time to populate the plain-data Ffmpeg struct
    let _folder_chooser_native = folder_chooser_native.clone();
    let _filename_entry = filename_entry.clone();
    let _format_chooser_combobox = format_chooser_combobox.clone();
    let _audio_source_combobox = audio_source_combobox.clone();
    let _audio_bitrate_spin = audio_bitrate_spin.clone();
    let _delay_spin2 = delay_spin.clone();
    let _frames_spin = frames_spin.clone();
    let _video_bitrate_spin = video_bitrate_spin.clone();
    let _follow_mouse_switch2 = follow_mouse_switch.clone();
    let _mouse_switch2 = mouse_switch.clone();
    let _area_switch2 = area_switch.clone();
    let second_click: Rc<RefCell<RecordClick>> = Rc::new(RefCell::new(RecordClick {
        is_record_button_clicked: false,
    }));
    record_button.connect_clicked(move |_| {
        let mode: RecordMode = if _area_grab_button.is_active() {
            RecordMode::Area
        } else if window_grab_button.is_active() {
            RecordMode::Window
        } else {
            RecordMode::Screen
        };
        if _area_grab_button.is_active() { _area_switch.set_sensitive(false); }
        if _video_switch.is_active() {
            _mouse_switch.set_sensitive(false);
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            _follow_mouse_switch.set_sensitive(false);
        }

        // Compute the output path from widgets (replaces the old get_filename()).
        let folder = _folder_chooser_native.file()
            .and_then(|f| f.path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let computed_filename = build_filename(
            &folder,
            &_filename_entry.text(),
            &_format_chooser_combobox.active_id().map(|s| s.to_string()).unwrap_or_default(),
        );
        if computed_filename.is_empty() || folder.is_empty() {
            if _area_grab_button.is_active() { _area_switch.set_sensitive(true); }
            if _video_switch.is_active() {
                _mouse_switch.set_sensitive(true);
                #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                _follow_mouse_switch.set_sensitive(true);
            }
            enable_input_widgets(_input_widgets.clone());
            _record_button.show();
            _record_time_label.set_visible(false);
            _stop_button.hide();
            stop_timer(_record_time_label.clone());
            return;
        }

        // Push current widget values into the plain-data struct before recording.
        {
            let mut rec = _ffmpeg_record_interface.borrow_mut();
            rec.filename             = computed_filename.clone();
            rec.audio_input_id       = _audio_source_combobox.active_id()
                                           .map(|s| s.to_string()).unwrap_or_default();
            rec.audio_record_bitrate = _audio_bitrate_spin.value() as u16;
            rec.record_delay         = _delay_spin2.value() as u16;
            rec.record_frames        = _frames_spin.value() as u16;
            rec.video_record_bitrate = _video_bitrate_spin.value() as u16;
            rec.audio_input_enabled  = _audio_input_switch.is_active();
            rec.audio_output_enabled = _audio_output_switch.is_active();
            rec.follow_mouse         = _follow_mouse_switch2.is_active();
            rec.record_mouse         = _mouse_switch2.is_active();
            rec.show_area            = _area_switch2.is_active();
            rec.video_enabled        = _video_switch.is_active();
        }

        if !_audio_input_switch.is_active() &&
            !_audio_output_switch.is_active() &&
            !_video_switch.is_active() ||
            !second_click.borrow_mut().is_clicked() &&
            _delay_spin.value() as u16 == 0 &&
            !is_overwrite(&get_bundle("already-exist", None),
                          &computed_filename,
                          _main_window.clone())
        {
            // Do nothing
        } else {
            _delay_window_button.set_active(false);
            if _delay_spin.value() as u16 > 0 {
                if !is_overwrite(&get_bundle("already-exist", None),
                                 &computed_filename,
                                 _main_window.clone())
                {
                    //Do nothing
                } else {
                    recording_delay(
                        _delay_spin.clone(),
                        _delay_spin.value() as u16,
                        delay_window.clone(),
                        _delay_window_button.clone(),
                        delay_window_label.clone(),
                        _record_button.clone(),
                        second_click.clone(),
                    );
                }
            } else if _delay_spin.value() as u16 == 0 {
                let _area_capture = area_capture.borrow_mut();
                #[cfg(target_os = "windows")]
                let _window_title = window_title.borrow_mut();
                disable_input_widgets(_input_widgets.clone());
                start_timer(_record_time_label.clone());
                _record_time_label.set_visible(true);
                if hide_switch.is_active() {
                    _main_window.minimize();
                }
                _play_button.hide();
                _record_button.hide();
                _stop_button.show();
                _main_window.set_deletable(false);
                if _audio_input_switch.is_active() && !_video_switch.is_active() {
                    match _ffmpeg_record_interface.borrow_mut().start_input_audio() {
                        Ok(_) => {
                            // Do nothing
                        },
                        Err(error) => {
                            if _area_grab_button.is_active() {
                                _area_switch.set_sensitive(true);
                            }
                            if _video_switch.is_active() {
                                _mouse_switch.set_sensitive(true);
                                #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                                _follow_mouse_switch.set_sensitive(true);
                            }
                            enable_input_widgets(_input_widgets.clone());
                            _record_button.show();
                            _record_time_label.set_visible(false);
                            _stop_button.hide();
                            stop_timer(_record_time_label.clone());
                            let text_buffer = TextBuffer::new(None);
                            text_buffer.set_text(&format!("{}", error));
                            _error_message.set_buffer(Some(&text_buffer));
                            _error_dialog.show();
                        },
                    }
                }
                if _audio_output_switch.is_active() && !_audio_input_switch.is_active()  && !_video_switch.is_active() {
                    match _ffmpeg_record_interface.borrow_mut().start_output_audio() {
                        Ok(_) => {
                            // Do nothing
                        },
                        Err(error) => {
                            if _area_grab_button.is_active() {
                                _area_switch.set_sensitive(true);
                            }
                            if _video_switch.is_active() {
                                _mouse_switch.set_sensitive(true);
                                #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                                _follow_mouse_switch.set_sensitive(true);
                            }
                            enable_input_widgets(_input_widgets.clone());
                            _record_button.show();
                            _record_time_label.set_visible(false);
                            _stop_button.hide();
                            stop_timer(_record_time_label.clone());
                            let text_buffer = TextBuffer::new(None);
                            text_buffer.set_text(&format!("{}", error));
                            _error_message.set_buffer(Some(&text_buffer));
                            _error_dialog.show();
                        },
                    }
                }
                if _video_switch.is_active() {
                    #[cfg(target_os = "windows")]
                    let start_video = _ffmpeg_record_interface.borrow_mut().start_video(
                        _area_capture.x,
                        _area_capture.y,
                        _area_capture.width,
                        _area_capture.height,
                        mode,
                        _window_title.title.clone(),
                    );
                    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                    let start_video = _ffmpeg_record_interface.borrow_mut().start_video(
                        _area_capture.x,
                        _area_capture.y,
                        _area_capture.width,
                        _area_capture.height,
                        mode
                    );
                    match start_video {
                        Ok(_) => {
                            // Do nothing
                        },
                        Err(error) => {
                            if _area_grab_button.is_active() {
                                _area_switch.set_sensitive(true);
                            }
                            if _video_switch.is_active() {
                                _mouse_switch.set_sensitive(true);
                                #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                                _follow_mouse_switch.set_sensitive(true);
                            }
                            enable_input_widgets(_input_widgets.clone());
                            _record_button.show();
                            _record_time_label.set_visible(false);
                            _stop_button.hide();
                            stop_timer(_record_time_label.clone());
                            // "__cancelled__" means the user dismissed the portal
                            // picker — not an error, so don't show the error dialog.
                            if error.to_string() != "__cancelled__" {
                                let text_buffer = TextBuffer::new(None);
                                text_buffer.set_text(&format!("{}", error));
                                _error_message.set_buffer(Some(&text_buffer));
                                _error_dialog.show();
                            }
                        },
                    }
                }
            }
        }
    });

    // Stop record button
    processing_label.set_label(&get_bundle("spinner-label", None));
    processing_label.set_wrap(true);
    processing_label.set_max_width_chars(16);
    processing_label.set_justify(adw::gtk::Justification::Center);
    stop_button.set_tooltip_text(Some(&get_bundle("stop-tooltip", None)));
    stop_label.set_label(&get_bundle("stop-recording", None));
    let _audio_input_switch = audio_input_switch.clone();
    let _audio_output_switch = audio_output_switch.clone();
    let _error_dialog = error_dialog.clone();
    let _error_message = error_message.clone();
    let _follow_mouse_switch = follow_mouse_switch.clone();
    let _mouse_switch = mouse_switch.clone();
    let _play_button = play_button.clone();
    let _record_button = record_button.clone();
    let _app_title = app_title.clone();
    let _processing_box = processing_box.clone();
    let _stop_button = stop_button.clone();
    let _video_switch = video_switch.clone();
    let _main_window_stop = main_window.clone();
    let mut _ffmpeg_record_interface = ffmpeg_record_interface.clone();
    stop_button.connect_clicked(move |button| {
        button.set_sensitive(false);
        _app_title.hide();
        _processing_box.show();
        let mut show_play = true;
        record_time_label.set_visible(false);
        stop_timer(record_time_label.clone());
        if _audio_input_switch.is_active() && !_video_switch.is_active() {
            match _ffmpeg_record_interface.borrow_mut().stop_input_audio() {
                Ok(_) => {},
                Err(error) => {
                    if area_grab_button.is_active() { area_switch.set_sensitive(true); }
                    if _video_switch.is_active() {
                        _mouse_switch.set_sensitive(true);
                        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                        _follow_mouse_switch.set_sensitive(true);
                    }
                    enable_input_widgets(input_widgets.clone());
                    _record_button.show();
                    show_play = false;
                    _stop_button.hide();
                    let text_buffer = TextBuffer::new(None);
                    text_buffer.set_text(&format!("{}", error));
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                },
            }
        }
        if _audio_output_switch.is_active() && !_audio_input_switch.is_active() && !_video_switch.is_active() {
            match _ffmpeg_record_interface.borrow_mut().stop_output_audio() {
                Ok(_) => {},
                Err(error) => {
                    if area_grab_button.is_active() { area_switch.set_sensitive(true); }
                    if _video_switch.is_active() {
                        _mouse_switch.set_sensitive(true);
                        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                        _follow_mouse_switch.set_sensitive(true);
                    }
                    enable_input_widgets(input_widgets.clone());
                    _record_button.show();
                    show_play = false;
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
                Ok(_) => {},
                Err(error) => {
                    if area_grab_button.is_active() { area_switch.set_sensitive(true); }
                    if _video_switch.is_active() {
                        _mouse_switch.set_sensitive(true);
                        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                        _follow_mouse_switch.set_sensitive(true);
                    }
                    enable_input_widgets(input_widgets.clone());
                    _record_button.show();
                    show_play = false;
                    _stop_button.hide();
                    let text_buffer = TextBuffer::new(None);
                    text_buffer.set_text(&format!("{}", error));
                    _error_message.set_buffer(Some(&text_buffer));
                    _error_dialog.show();
                },
            }
        }
        if area_grab_button.is_active() { area_switch.set_sensitive(true); }
        if _video_switch.is_active() {
            _mouse_switch.set_sensitive(true);
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            _follow_mouse_switch.set_sensitive(true);
        }
        enable_input_widgets(input_widgets.clone());
        if show_play {
            let file_name = _ffmpeg_record_interface.borrow_mut().saved_filename.clone();
            if !file_name.is_empty() && std::path::Path::new(&file_name).exists() {
                _play_button.set_tooltip_text(Some(&get_bundle("play-tooltip", None)));
                _play_button.show();
            }
        }
        let stop_button = _stop_button.clone();
        let record_button = _record_button.clone();
        let processing_box = _processing_box.clone();
        let app_title = _app_title.clone();
        let main_window = _main_window_stop.clone();
        glib::idle_add_local_once(move || {
            processing_box.hide();
            app_title.show();
            stop_button.hide();
            record_button.show();
            main_window.set_deletable(true);
        });
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

    main_window.connect_close_request(move |win| {
        ffmpeg_record_interface.borrow_mut().kill().unwrap();
        win.destroy();
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
    adw::gtk::StyleContext::add_provider_for_display(
        &display,
        &grab_color_provider,
        adw::gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    main_window.show();

    Ok(())
}
