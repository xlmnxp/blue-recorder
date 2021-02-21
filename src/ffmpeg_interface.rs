extern crate subprocess;
use chrono::prelude::*;
use gtk::{
    CheckButton, ComboBoxExt, ComboBoxText, Entry, EntryExt, FileChooser, FileChooserExt,
    SpinButton, SpinButtonExt, ToggleButtonExt,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
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
    pub unbound: Option<Sender<bool>>,
}

impl Ffmpeg {
    pub fn start_record(&mut self, x: u16, y: u16, width: u16, height: u16) -> u32 {
        if self.process_id.is_some() {
            self.stop_record();
        }

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
        if is_wayland() {
            if self.record_video.get_active() {
                if self.unbound.is_some() {
                    self.clone()
                        .unbound
                        .unwrap()
                        .send(false)
                        .unwrap_or_default();
                }
                self.record_wayland(
                    format!("{}.temp", self.saved_filename.as_ref().unwrap().to_string()),
                    x,
                    y,
                    width,
                    height,
                );
            }

            if self.record_audio.get_active() {
                let mut ffmpeg_command = Command::new("ffmpeg");
                ffmpeg_command.arg("-f");
                ffmpeg_command.arg("pulse");
                ffmpeg_command.arg("-i");
                ffmpeg_command.arg(self.audio_id.get_active_id().unwrap().to_string());
                ffmpeg_command.arg("-f");
                ffmpeg_command.arg("ogg");
                ffmpeg_command.arg(format!(
                    "{}.temp.audio",
                    self.saved_filename.as_ref().unwrap().to_string()
                ));
                ffmpeg_command.arg("-y");
                self.process_id = Some(ffmpeg_command.spawn().unwrap().id());
            }
            return 0;
        }

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

        ffmpeg_command.arg(self.saved_filename.as_ref().unwrap());
        ffmpeg_command.arg("-y");

        // sleep for delay
        sleep(Duration::from_secs(self.record_delay.get_value() as u64));

        // start recording and return the process id
        self.process_id = Some(ffmpeg_command.spawn().unwrap().id());
        self.process_id.unwrap()
    }

    pub fn stop_record(&self) {
        // kill the process to stop recording
        if self.process_id.is_some() {
            Command::new("kill")
                .arg(format!("{}", self.process_id.unwrap()))
                .output()
                .unwrap();
        }

        if is_wayland() {
            // create new dbus session
            let connection = zbus::Connection::new_session().unwrap();
            // bind the connection to gnome screencast proxy
            let gnome_screencast_proxy = GnomeScreencastProxy::new(&connection).unwrap();
            gnome_screencast_proxy.stop_screencast().unwrap();
            let is_audio_record = std::path::Path::new(
                format!("{}.temp.audio", self.saved_filename.as_ref().unwrap()).as_str(),
            )
            .exists();
            let is_video_record = std::path::Path::new(
                format!("{}.temp", self.saved_filename.as_ref().unwrap()).as_str(),
            )
            .exists();
            if self.unbound.is_some() {
                self.unbound
                    .as_ref()
                    .unwrap()
                    .send(true)
                    .unwrap_or_default();

                // convert webm to the format user choose using ffmpeg
                if is_video_record {
                    let mut ffmpeg_convert_command = Command::new("ffmpeg");
                    ffmpeg_convert_command.arg("-f");
                    ffmpeg_convert_command.arg("webm");
                    ffmpeg_convert_command.arg("-i");
                    ffmpeg_convert_command
                        .arg(format!("{}.temp", self.saved_filename.as_ref().unwrap()));
                    ffmpeg_convert_command.arg(format!(
                        "{}{}",
                        self.saved_filename.as_ref().unwrap(),
                        if is_audio_record {
                            format!(
                                ".temp.without.audio.{}",
                                self.filename.2.get_active_id().unwrap().to_string()
                            )
                        } else {
                            "".to_string()
                        }
                    ));
                    ffmpeg_convert_command.arg("-y");
                    ffmpeg_convert_command.output().unwrap();
                    std::fs::remove_file(format!("{}.temp", self.saved_filename.as_ref().unwrap()))
                        .unwrap();

                    if is_audio_record {
                        // merge audio with video
                        let mut ffmpeg_audio_merge_command = Command::new("ffmpeg");
                        ffmpeg_audio_merge_command.arg("-i");
                        ffmpeg_audio_merge_command.arg(format!(
                            "{}.temp.without.audio.{}",
                            self.saved_filename.as_ref().unwrap(),
                            self.filename.2.get_active_id().unwrap().to_string()
                        ));
                        ffmpeg_audio_merge_command.arg("-i");
                        ffmpeg_audio_merge_command.arg(format!(
                            "{}.temp.audio",
                            self.saved_filename.as_ref().unwrap()
                        ));
                        ffmpeg_audio_merge_command.arg("-c:v");
                        ffmpeg_audio_merge_command.arg("copy");
                        ffmpeg_audio_merge_command.arg("-c:a");
                        ffmpeg_audio_merge_command.arg("aac");
                        ffmpeg_audio_merge_command.arg(self.saved_filename.as_ref().unwrap());
                        ffmpeg_audio_merge_command.arg("-y");
                        ffmpeg_audio_merge_command.output().unwrap();
                        std::fs::remove_file(format!(
                            "{}.temp.audio",
                            self.saved_filename.as_ref().unwrap()
                        ))
                        .unwrap();
                        std::fs::remove_file(format!(
                            "{}.temp.without.audio.{}",
                            self.saved_filename.as_ref().unwrap(),
                            self.filename.2.get_active_id().unwrap().to_string()
                        ))
                        .unwrap();
                    }
                }
            } else if is_audio_record {
                println!("convert audio");
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("ogg")
                    .arg("-i")
                    .arg(format!(
                        "{}.temp.audio",
                        self.saved_filename.as_ref().unwrap()
                    ))
                    .arg(format!("{}", self.saved_filename.as_ref().unwrap()))
                    .output()
                    .unwrap();
                std::fs::remove_file(format!(
                    "{}.temp.audio",
                    self.saved_filename.as_ref().unwrap()
                ))
                .unwrap();
            }
        }

        // execute command after finish recording
        if !(self.command.get_text().trim() == "") {
            Exec::shell(self.command.get_text().trim()).popen().unwrap();
        }
    }

    // Gnome screencast for record wayland
    pub fn record_wayland(&mut self, filename: String, x: u16, y: u16, width: u16, height: u16) {
        // create new dbus session
        let connection = zbus::Connection::new_session().unwrap();
        // bind the connection to gnome screencast proxy
        let gnome_screencast_proxy = GnomeScreencastProxy::new(&connection).unwrap();
        // options for gnome screencast
        let mut screencast_options: HashMap<&str, Value> = HashMap::new();
        screencast_options.insert("framerate", Value::new(self.record_frames.get_value()));
        screencast_options.insert("draw-cursor", Value::new(self.record_mouse.get_active()));
        screencast_options.insert("pipeline", Value::new("vp8enc min_quantizer=10 max_quantizer=50 cq_level=13 cpu-used=5 deadline=1000000 threads=%T ! queue ! webmmux"));
        // make unbound channel for communication with record thread
        let (tx, tr): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        self.unbound = Some(tx);
        let receiver: Receiver<bool> = tr;

        // start recording in another thread
        std::thread::spawn(move || {
            gnome_screencast_proxy
                .screencast_area(
                    x.into(),
                    y.into(),
                    width.into(),
                    height.into(),
                    &filename,
                    screencast_options,
                )
                .unwrap();

            loop {
                if receiver.recv().unwrap_or(false) {
                    break;
                }
            }
        });
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

fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap()
        .eq_ignore_ascii_case("wayland")
}
