extern crate gio;
extern crate gtk;
mod config_management;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{
    Adjustment, Builder, Button, CheckButton, ComboBox, Entry, FileChooser, Label, MenuItem,
    SpinButton, Window, Dialog
};
use std::path::Path;
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let builder: Builder = Builder::from_file(Path::new("windows/ui.glade"));

    config_management::initialize();
    config_management::set("default", "frame", "30");
    // get Objects from UI
    let main_window: Window = builder.get_object("window1").unwrap();
    let about_dialog: Dialog = builder.get_object("aboutdialog").unwrap();
    let area_chooser: Window = builder.get_object("window2").unwrap();
    let folder_chooser: FileChooser = builder.get_object("filechooser").unwrap();
    let filename_entry: Entry = builder.get_object("filename").unwrap();
    let command_entry: Entry = builder.get_object("command").unwrap();
    let format_chooser: ComboBox = builder.get_object("comboboxtext1").unwrap();
    let audio_source: ComboBox = builder.get_object("audiosource").unwrap();
    let record_button: Button = builder.get_object("recordbutton").unwrap();
    let stop_button: Button = builder.get_object("stopbutton").unwrap();
    let window_grab_button: Button = builder.get_object("button4").unwrap();
    let area_grab_button: Button = builder.get_object("button5").unwrap();
    let frame_text: Label = builder.get_object("label2").unwrap();
    let delay_text: Label = builder.get_object("label3").unwrap();
    let command_text: Label = builder.get_object("label6").unwrap();
    let frames_spin: SpinButton = builder.get_object("frames").unwrap();
    let delay_spin: SpinButton = builder.get_object("delay").unwrap();
    let audio_source_label: Label = builder.get_object("audiosourcelabel").unwrap();
    let delay_adjustment: Adjustment = builder.get_object("adjustment1").unwrap();
    let frames_adjustment: Adjustment = builder.get_object("adjustment2").unwrap();
    let delay_pref_adjustment: Adjustment = builder.get_object("adjustment3").unwrap();
    let play_button: Button = builder.get_object("playbutton").unwrap();
    let video_switch: CheckButton = builder.get_object("videoswitch").unwrap();
    let audio_switch: CheckButton = builder.get_object("audioswitch").unwrap();
    let mouse_switch: CheckButton = builder.get_object("mouseswitch").unwrap();
    let follow_mouse_switch: CheckButton = builder.get_object("followmouseswitch").unwrap();
    let about_menu_item: MenuItem = builder.get_object("item2").unwrap();
    
    // --- default properties
    about_menu_item.set_label("about");


    // --- connections
    // show dialog window when about button clicked then hide it after close 
    about_menu_item.connect_activate(move |_| {
        &about_dialog.run();
        &about_dialog.hide();
    });
    // close the application when main window destroy
    main_window.connect_destroy(|_| {
        std::process::exit(0);
    });
    gtk::main();
}
