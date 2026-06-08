pub mod area_capture;
pub mod config_management;
pub mod fluent;
pub mod timer;
pub mod utils;
mod ui;

use adw::Application;
use adw::prelude::{ApplicationExt, ApplicationExtManual};
use ui::run_ui;

#[async_std::main]
async fn main() {
    // Inside the snap sandbox, XDG_RUNTIME_DIR is rewritten to a snap-private
    // subdirectory, but the host's PipeWire socket stays in the parent directory.
    // Point PipeWire at the real socket location so pipewiresrc can connect.
    if std::env::var_os("SNAP").is_some() {
        if let Some(xdg_runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            if let Some(parent) = std::path::Path::new(&xdg_runtime_dir).parent() {
                if parent.join("pipewire-0").exists() {
                    unsafe {
                        std::env::set_var("PIPEWIRE_RUNTIME_DIR", parent);
                    }
                }
            }
        }
    }

    // Init GTK
    adw::gtk::init().expect("Failed to initialize GTK.");

    // Create new application
    let application = Application::new(Some("sa.sy.blue-recorder"), Default::default());
    application.connect_activate(run_ui);
    application.run();
}
