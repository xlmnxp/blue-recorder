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

    // fatch and make the config file
    if !&config_path.exists() {
        let config_directories = &mut config_path.to_owned();
        config_directories.pop();
        std::fs::create_dir_all(&config_directories).unwrap_or_default();
        std::fs::File::create(&config_path).unwrap();
        default();
    }

    config_path
}

pub fn merge_previous_version() -> Option<PathBuf> {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");

    // return none if config.ini not exists
    if !&config_path.exists() {
        return None;
    }

    let mut config_string: String = String::from_utf8(std::fs::read(&config_path).unwrap()).unwrap();
    config_string = config_string.replace("Options", "default").replace("True", "1").replace("False", "0");
    std::fs::write(&config_path, config_string).unwrap();
    Some(config_path)
}

fn default() {
    set("default", "frame", "50");
    set("default", "delay", "0");
    set(
        "default",
        "folder",
        String::from("file://")
            .add(
                glib::get_user_special_dir(glib::UserDirectory::Videos)
                    .expect(std::env::var("HOME").expect("/").as_str())
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
    get(&selection, &key).eq_ignore_ascii_case("1")
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
    set(&selection, &key, if value { "1" } else { "0" })
}
