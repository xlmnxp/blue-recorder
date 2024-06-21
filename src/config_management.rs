extern crate dirs;
extern crate glib;
extern crate ini;

use glib::get_user_data_dir;
use ini::Ini;
use std::ops::Add;
use std::path::{Path, PathBuf};

pub fn initialize() -> PathBuf {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");

    // Fatch and make the config file
    if !&config_path.exists() {
        let config_directories = &mut config_path.to_owned();
        config_directories.pop();
        std::fs::create_dir_all(&config_directories).unwrap_or_default();
        std::fs::File::create(&config_path).unwrap();
        default();
    } else {
        merge_previous_version();
    }

    config_path
}

fn default() {
    for format in 0..7 {
        set_default_video_bitrate(&format.to_string());
        set_default_frame(&format.to_string());
    }
    set("default", "delay", "0");
    set("default", "format", "0");
    set(
        "default",
        "folder",
        String::from("file://")
            .add(
                glib::get_user_special_dir(glib::UserDirectory::Videos)
                    .unwrap_or_else(|| {
                        PathBuf::from(
                            std::env::var("HOME")
                                .unwrap_or_else(|_| "/".to_string())
                                .as_str(),
                        )
                    })
                    .to_str()
                    .unwrap(),
            )
            .as_str(),
    );
    set("default", "command", "");
    set("default", "filename", "");
    set("default", "videocheck", "1");
    set("default", "audiocheck", "1");
    set("default", "mousecheck", "1");
    set("default", "followmousecheck", "0");
    set("default", "hidecheck", "0");
    set("default", "speakercheck", "0");
    set("default", "mode", "screen");
}

fn merge_previous_version() -> Option<PathBuf> {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");

    // Return none if config.ini not exists
    if !&config_path.exists() {
        return None;
    }

    let mut config_string: String =
        String::from_utf8(std::fs::read(&config_path).unwrap()).unwrap();
    config_string = config_string
        .replace("Options", "default")
        .replace("True", "1")
        .replace("False", "0");
    std::fs::write(&config_path, config_string).unwrap();
    Some(config_path)
}

pub fn get(selection: &str, key: &str) -> String {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");
    String::from(
        Ini::load_from_file(&config_path)
            .unwrap()
            .with_section(Some(selection))
            .get(key)
            .unwrap_or_default(),
    )
}

pub fn get_bool(selection: &str, key: &str) -> bool {
    get(selection, key).eq_ignore_ascii_case("1")
}

pub fn set(selection: &str, key: &str, value: &str) -> bool {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");
    let mut config_init = Ini::load_from_file(&config_path).unwrap_or_default();
    config_init.with_section(Some(selection)).set(key, value);
    config_init.write_to_file(&config_path).is_ok()
}

pub fn set_bool(selection: &str, key: &str, value: bool) -> bool {
    set(selection, key, if value { "1" } else { "0" })
}

pub fn folder_icon(folder_chooser_name: Option<&str>) -> &str {
    let home_folder = dirs::home_dir().unwrap();
    if folder_chooser_name == home_folder.as_path().file_name().unwrap().to_str() {
        "user-home"
    } else {
        match folder_chooser_name {
            Some("/") => "drive-harddisk",
            Some("Desktop") => "user-desktop",
            Some("Documents") => "folder-documents",
            Some("Downloads") => "folder-download",
            Some("Music") => "folder-music",
            Some("Pictures") => "folder-pictures",
            Some("Public") => "folder-publicshare",
            Some("Templates") => "folder-templates",
            Some("Videos") => "folder-videos",
            _ => "folder",
        }
    }
}

pub fn set_default_video_bitrate(format: &str) -> bool {
    let rate = match format {
        "0" => self::set("default", "videobitrate-0", "0"),
        "1" => self::set("default", "videobitrate-1", "0"),
        "2" => self::set("default", "videobitrate-2", "0"),
        "3" => self::set("default", "videobitrate-3", "0"),
        "4" => self::set("default", "videobitrate-4", "0"),
        "5" => self::set("default", "videobitrate-5", "0"),
        "6" => self::set("default", "videobitrate-6", "0"),
        _ => self::set("default", "videobitrate-0", "0"), // Default value (disabled)
    };
    rate
}

pub fn set_default_frame(format: &str) -> bool {
    let rate = match format {
        "0" => self::set("default", "frame-0", "60"),
        "1" => self::set("default", "frame-1", "60"),
        "2" => self::set("default", "frame-2", "60"),
        "3" => self::set("default", "frame-3", "10"),
        "4" => self::set("default", "frame-4", "2"),
        "5" => self::set("default", "frame-5", "2"),
        "6" => self::set("default", "frame-6", "0"),
        _ => self::set("default", "frame-0", "0"), // Default value (disabled)
    };
    rate
}
