//use gtk::Button;
//use gtk::prelude::*;
use ksni::menu::StandardItem;
use ksni::Tray;
use std::path::Path;

pub struct BlueRecorderTray {
    //pub stop_record_button: Button,
}

impl Tray for BlueRecorderTray {
    fn icon_theme_path(&self) -> String {
        let mut indicator_icon_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }
    .join(Path::new("data/"));

    if !indicator_icon_path.exists() {
        indicator_icon_path = std::fs::canonicalize(Path::new(
            &std::env::var("DATA_DIR")
                .unwrap_or_else(|_| String::from("data/"))
        ))
        .unwrap();
    }
        indicator_icon_path.to_str().unwrap().into()
     }

   fn icon_name(&self) -> String {
        "blue-recorder-active".into()
    }

    fn title(&self) -> String {
        "Recording".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Stop Recording".into(),
                icon_name: "media-playback-stop".into(),
                //activate: Box::new(|menu_button: &mut Self| {
                    //menu_button.stop_record_button.emit_clicked();
                //}),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayService {
  tray_handle: ksni::Handle<BlueRecorderTray>,
}

impl TrayService {
    pub fn show() -> Self {
        let service = ksni::TrayService::new(BlueRecorderTray{});
        let tray_handle = service.handle();
        service.spawn();
        TrayService { tray_handle }
    }

    pub fn close(&self) {
        self.tray_handle.shutdown();
    }
}

