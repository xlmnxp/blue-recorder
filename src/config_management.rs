extern crate ini;
extern crate glib;

use glib::get_user_data_dir;
use ini::Ini;
use std::path::{Path, PathBuf};
use std::ops::Add;

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
                    .to_str().unwrap(),
            )
            .as_str(),
    );
    set("default", "command", "");
    set("default", "filename", "");
    set("default", "videocheck", "true");
    set("default", "audiocheck", "true");
    set("default", "mousecheck", "true");
    set("default", "followmousecheck", "false");
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

pub fn set(selection: &str, key: &str, value: &str) -> bool {
    let config_path: PathBuf = Path::new(&get_user_data_dir().unwrap())
        .join("blue-recorder")
        .join("config.ini");
    let mut config_init = Ini::load_from_file(&config_path).unwrap_or_default();
    config_init.with_section(Some(selection)).set(key, value);
    config_init.write_to_file(&config_path).is_ok()
}
