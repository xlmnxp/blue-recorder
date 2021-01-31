extern crate gio;
extern crate gtk;
extern crate glib;

use std::path::PathBuf;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Builder, Window};
use glib::get_user_data_dir;
use std::path::Path;
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let builder: Builder = Builder::from_file(Path::new("windows/ui.glade"));
    
    // fatch and make the config file
    let config_path = Path::new(&get_user_data_dir().unwrap()).join("blue-recorder").join("config.ini");
    if !&config_path.exists() {
        let config_directories = &mut config_path.to_owned();
        config_directories.pop();
        std::fs::create_dir_all(&config_directories).unwrap_or_default();
        std::fs::File::create(&config_path).unwrap();
    }

    // get Objects from UI
    let main_window: Window = builder.get_object("window1").unwrap();

    // close the application when main window destroy
    main_window.connect_destroy(|_| {
        std::process::exit(0);
    });
    gtk::main();
}
