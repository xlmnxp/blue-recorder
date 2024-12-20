pub mod area_capture;
pub mod config_management;
pub mod fluent;
pub mod timer;
mod ui;

use libadwaita::Application;
use libadwaita::prelude::{ApplicationExt, ApplicationExtManual};
use ui::run_ui;

#[async_std::main]
async fn main() {
    // Init GTK
    libadwaita::gtk::init().expect("Failed to initialize GTK.");

    // Create new application
    let application = Application::new(None, Default::default());
    application.connect_activate(run_ui);
    application.run();
}
