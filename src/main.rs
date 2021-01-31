extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Builder, Window};
use std::path::Path;
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let builder: Builder = Builder::from_file(Path::new("windows/ui.glade"));
    let main_window: Window = builder.get_object("window1").unwrap();

    main_window.connect_destroy(|_| {
        std::process::exit(0);
    });
    gtk::main();
}
