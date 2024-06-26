use fluent_bundle::bundle::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use gtk::prelude::{ButtonExt, GtkWindowExt, WidgetExt};
use gtk::{Box, Button, ButtonsType, DialogFlags, MessageDialog, MessageType, Orientation, Window};
use std::path::Path;

pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .eq_ignore_ascii_case("wayland")
}

pub fn is_snap() -> bool {
    !std::env::var("SNAP").unwrap_or_default().is_empty()
}

pub enum BlueRecorderError {
    Start,
    Stop,
    Play,
}

pub fn error_message(error_type: BlueRecorderError) -> String {
    let error_message = match error_type {
        BlueRecorderError::Start => get_bundle("start-error", None),
        BlueRecorderError::Stop => get_bundle("stop-error", None),
        BlueRecorderError::Play => get_bundle("play-error", None),
    };
    error_message
}

// Translate
pub fn get_bundle(message_id: &str, arg: Option<&FluentArgs>) -> String {
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
    bundle.format_pattern(bundle.get_message(message_id)
                          .unwrap().value().unwrap(), arg, &mut vec![]).to_string()
}

pub fn show_error_dialog(message: String, window: Window) {
    let error_dialog = MessageDialog::new(
        Some(&window),
        DialogFlags::all(),
        MessageType::Error,
        ButtonsType::Close,
        &message,
    );
    let dialog_box = Box::new(Orientation::Horizontal, 10);
    let details_button = Button::new();
    details_button.set_label(&get_bundle("details-button", None));
    error_dialog.set_child(Some(&details_button));
    error_dialog.show();
}
