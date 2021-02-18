extern crate subprocess;
// extern crate dbus;
use chrono::prelude::*;
use gtk::{
    CheckButton, ComboBoxExt, ComboBoxText, Entry, EntryExt, FileChooser, FileChooserExt,
    SpinButton, SpinButtonExt, ToggleButtonExt,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use subprocess::Exec;
use zbus::dbus_proxy;
use zvariant::Value;

trait GnomeScreencastResult {}

#[dbus_proxy(
    interface = "org.gnome.Shell.Screencast",
    default_path = "/org/gnome/Shell/Screencast"
)]
trait GnomeScreencast {
    fn screencast(
        &self,
        file_template: &str,
        options: HashMap<&str, Value>,
    ) -> zbus::Result<(bool, String)>;

    fn screencast_area(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        file_template: &str,
        options: HashMap<&str, Value>,
    ) -> zbus::Result<(bool, String)>;
    fn stop_screencast(&self) -> zbus::Result<bool>;
}

#[derive(Clone)]
pub struct Ffmpeg {
    pub filename: (FileChooser, Entry, ComboBoxText),
    pub record_video: CheckButton,
    pub record_audio: CheckButton,
    pub audio_id: ComboBoxText,
    pub record_mouse: CheckButton,
    pub follow_mouse: CheckButton,
    pub record_frames: SpinButton,
    pub record_delay: SpinButton,
    pub command: Entry,
    pub process_id: Option<u32>,
    pub saved_filename: Option<String>,
}

impl Ffmpeg {
    pub fn start_record(&mut self, x: u16, y: u16, width: u16, height: u16) -> u32 {
        if self.process_id.is_some() {
            self.clone().stop_record();
        }
        // self.clone().record_wayland();

        let mut ffmpeg_command: Command = Command::new("ffmpeg");

        // if recorder video switch is enabled, record video with specified width and hight
        if self.record_video.get_active() {
            ffmpeg_command.arg("-video_size");
            ffmpeg_command.arg(format!("{}x{}", width, height));
        }

        // if show mouse switch is enabled, draw the mouse to video
        ffmpeg_command.arg("-draw_mouse");
        if self.record_mouse.get_active() {
            ffmpeg_command.arg("1");
        } else {
            ffmpeg_command.arg("0");
        }

        // if follow mouse switch is enabled, follow the mouse
        if self.follow_mouse.get_active() {
            ffmpeg_command.arg("-follow_mouse");
            ffmpeg_command.arg("centered");
        }

        ffmpeg_command.arg("-framerate");
        ffmpeg_command.arg(format!("{}", self.record_frames.get_value()));
        ffmpeg_command.arg("-f");
        ffmpeg_command.arg("x11grab");
        ffmpeg_command.arg("-i");
        ffmpeg_command.arg(format!(
            "{}+{},{}",
            std::env::var("DISPLAY").expect(":1").as_str(),
            x,
            y
        ));

        // if follow audio switch is enabled, record the audio
        if self.record_audio.get_active() {
            ffmpeg_command.arg("-f");
            ffmpeg_command.arg("pulse");
            ffmpeg_command.arg("-i");
            ffmpeg_command.arg(self.audio_id.get_active_id().unwrap().to_string());
            ffmpeg_command.arg("-strict");
            ffmpeg_command.arg("-2");
        }

        ffmpeg_command.arg("-q");
        ffmpeg_command.arg("1");

        self.saved_filename = Some(
            self.filename
                .0
                .get_filename()
                .unwrap()
                .join(PathBuf::from(format!(
                    "{}.{}",
                    if self.filename.1.get_text().to_string().trim().eq("") {
                        Utc::now().to_string().replace(" UTC", "")
                    } else {
                        self.filename.1.get_text().to_string().trim().to_string()
                    },
                    self.filename.2.get_active_id().unwrap().to_string()
                )))
                .as_path()
                .display()
                .to_string(),
        );

        ffmpeg_command.arg(self.saved_filename.as_ref().unwrap());
        ffmpeg_command.arg("-y");

        // sleep for delay
        sleep(Duration::from_secs(self.record_delay.get_value() as u64));

        // start recording and return the process id
        self.process_id = Some(ffmpeg_command.spawn().unwrap().id());
        self.process_id.unwrap()
    }

    pub fn stop_record(self) {
        // kill the process to stop recording
        if self.process_id.is_some() {
            Command::new("kill")
                .arg(format!("{}", self.process_id.unwrap()))
                .output()
                .unwrap();
        }

        // execute command after finish recording
        if !(self.command.get_text().trim() == "") {
            Exec::shell(self.command.get_text().trim()).popen().unwrap();
        }
    }

    // Gnome screencast for record wayland
    pub fn record_wayland(self) {
        // create new dbus session
        let connection = zbus::Connection::new_session().unwrap();

        // bind the connection to gnome screencast proxy
        let gnome_screencast_proxy = GnomeScreencastProxy::new(&connection).unwrap();
        
        // options for gnome screencast
        let mut screencast_options: HashMap<&str, Value> = HashMap::new();
        screencast_options.insert("framerate", Value::new(self.record_frames.get_value()));
        screencast_options.insert("draw-cursor", Value::new(self.record_mouse.get_active()));
        gnome_screencast_proxy
            .screencast(
                "/home/xlmnxp/Projects/blue-recorder/test/test.webm",
                screencast_options,
            )
            .unwrap();

        // for testing it will record 20 seconds then stop the recording
        sleep(Duration::from_secs(20u64));
        gnome_screencast_proxy.stop_screencast().unwrap();
    }

    pub fn play_record(self) {
        if self.saved_filename.is_some() {
            Command::new("xdg-open")
                .arg(self.saved_filename.unwrap())
                .spawn()
                .unwrap();
        }
    }
}
