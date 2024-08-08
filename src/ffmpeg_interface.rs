extern crate subprocess;
use crate::utils::{is_snap, is_wayland};
use crate::wayland_record::{CursorModeTypes, RecordTypes, WaylandRecorder};
use chrono::prelude::*;
use filename::Filename;
use gettextrs::gettext;
use gtk::{prelude::*, ResponseType};
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
use gtk::{CheckButton, ComboBoxText, Entry, FileChooserNative, SpinButton, Window};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use subprocess::Exec;

#[derive(Clone)]
pub struct Ffmpeg {
    pub filename: (FileChooserNative, Entry, ComboBoxText),
    pub record_video: CheckButton,
    pub record_audio: CheckButton,
    pub audio_id: ComboBoxText,
    pub record_mouse: CheckButton,
    pub follow_mouse: CheckButton,
    pub record_frames: SpinButton,
    pub record_delay: SpinButton,
    pub command: Entry,
    pub video_process: Option<Rc<RefCell<Child>>>,
    pub audio_process: Option<Rc<RefCell<Child>>>,
    pub saved_filename: Option<String>,
    pub unbound: Option<Sender<bool>>,
    pub window: Window,
    pub record_wayland: WaylandRecorder,
    pub record_window: Rc<RefCell<bool>>,
    pub main_context: gtk::glib::MainContext,
    pub temp_video_filename: String,
}

impl Ffmpeg {
    pub fn start_record(&mut self, x: u16, y: u16, width: u16, height: u16) -> Option<()> {
        self.saved_filename = Some(
            self.filename
                .0
                .file()
                .unwrap()
                .path()
                .unwrap()
                .join(PathBuf::from(format!(
                    "{}.{}",
                    if self.filename.1.text().to_string().trim().eq("") {
                        Utc::now().to_string().replace(" UTC", "").replace(' ', "-")
                    } else {
                        self.filename.1.text().to_string().trim().to_string()
                    },
                    self.filename.2.active_id().unwrap()
                )))
                .as_path()
                .display()
                .to_string(),
        );

        let is_file_already_exists =
            std::path::Path::new(&self.saved_filename.clone().unwrap()).exists();

        if is_file_already_exists {
            let message_dialog = MessageDialog::new(
                Some(&self.window),
                DialogFlags::all(),
                MessageType::Warning,
                ButtonsType::YesNo,
                &gettext("File already exist. Do you want to overwrite it?"),
            );

            let answer = self.main_context.block_on(message_dialog.run_future());
            message_dialog.close();

            if answer != ResponseType::Yes {
                return None;
            }
        }

        if self.record_video.is_active() && !is_wayland() {
            let mut ffmpeg_command: Command = Command::new("ffmpeg");

            // record video with specified width and hight
            ffmpeg_command.args([
                "-video_size",
                format!("{}x{}", width, height).as_str(),
                "-framerate",
                self.record_frames.value().to_string().as_str(),
                "-f",
                "x11grab",
                "-i",
                format!(
                    "{}+{},{}",
                    std::env::var("DISPLAY")
                        .unwrap_or_else(|_| ":0".to_string())
                        .as_str(),
                    x,
                    y
                )
                .as_str(),
            ]);

            // if show mouse switch is enabled, draw the mouse to video
            ffmpeg_command.arg("-draw_mouse");
            if self.record_mouse.is_active() {
                ffmpeg_command.arg("1");
            } else {
                ffmpeg_command.arg("0");
            }

            // if follow mouse switch is enabled, follow the mouse
            if self.follow_mouse.is_active() {
                ffmpeg_command.args(["-follow_mouse", "centered"]);
            }

            let video_filename = format!(
                "{}.temp.without.audio.{}",
                self.saved_filename.as_ref().unwrap(),
                self.filename.2.active_id().unwrap()
            );

            ffmpeg_command.args([
                "-crf",
                "1",
                {
                    if self.record_audio.is_active() {
                        video_filename.as_str()
                    } else {
                        self.saved_filename.as_ref().unwrap()
                    }
                },
                "-y",
            ]);

            // sleep for delay
            sleep(Duration::from_secs(self.record_delay.value() as u64));

            // start recording and return the process id
            self.video_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn().unwrap())));
        } else if self.record_video.is_active() && is_wayland() {
            sleep(Duration::from_secs(self.record_delay.value() as u64));

            let tempfile = tempfile::NamedTempFile::new()
                .expect("cannot create temp file")
                .keep()
                .expect("cannot keep temp file");
            self.temp_video_filename = tempfile
                .0
                .file_name()
                .expect("cannot get file name")
                .to_str()
                .unwrap()
                .to_string();

            let record_window = self.record_window.take();
            self.record_window.replace(record_window);

            if !self.main_context.block_on(self.record_wayland.start(
                self.temp_video_filename.clone(),
                if record_window {
                    RecordTypes::Window
                } else {
                    RecordTypes::Monitor
                },
                {
                    if self.record_mouse.is_active() {
                        CursorModeTypes::Show
                    } else {
                        CursorModeTypes::Hidden
                    }
                },
            )) {
                println!("failed to start recording");
                return None;
            }
        }

        if self.record_audio.is_active() {
            let mut ffmpeg_command = Command::new("ffmpeg");
            ffmpeg_command.arg("-f");
            ffmpeg_command.arg("pulse");
            ffmpeg_command.arg("-i");
            ffmpeg_command.arg(&self.audio_id.active_id().unwrap());
            ffmpeg_command.arg("-f");
            ffmpeg_command.arg("ogg");
            ffmpeg_command.arg(format!(
                "{}.temp.audio",
                self.saved_filename.as_ref().unwrap()
            ));
            ffmpeg_command.arg("-y");
            self.audio_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn().unwrap())));
        }

        Some(())
    }

    pub fn stop_record(&mut self) {
        // kill the process to stop recording
        if self.video_process.is_some() {
            Command::new("kill")
                .arg(format!(
                    "{}",
                    self.video_process.clone().unwrap().borrow_mut().id()
                ))
                .output()
                .unwrap();

            self.video_process
                .clone()
                .unwrap()
                .borrow_mut()
                .wait()
                .unwrap();

            println!("video killed");
        } else if is_wayland() {
            self.main_context.block_on(self.record_wayland.stop());
        }

        if self.audio_process.is_some() {
            Command::new("kill")
                .arg(format!(
                    "{}",
                    self.audio_process.clone().unwrap().borrow_mut().id()
                ))
                .output()
                .unwrap();

            self.audio_process
                .clone()
                .unwrap()
                .borrow_mut()
                .wait()
                .unwrap();
            println!("audio killed");
        }

        let video_filename = {
            if is_wayland() {
                self.temp_video_filename.clone()
            } else {
                format!(
                    "{}.temp.without.audio.{}",
                    self.saved_filename.as_ref().unwrap(),
                    self.filename.2.active_id().unwrap()
                )
            }
        };

        let audio_filename = format!("{}.temp.audio", self.saved_filename.as_ref().unwrap());

        let is_video_record = { std::path::Path::new(video_filename.as_str()).exists() };
        let is_audio_record = std::path::Path::new(audio_filename.as_str()).exists();

        if is_video_record {
            if is_wayland() {
                // convert webm to specified format
                Command::new("ffmpeg")
                    .args([
                        "-i",
                        self.temp_video_filename.as_str(),
                        "-crf",
                        "23", // default quality
                        "-c:a",
                        self.filename.2.active_id().unwrap().as_str(),
                        self.saved_filename.as_ref().unwrap(),
                        "-y",
                    ])
                    .output()
                    .unwrap();
            } else {
                let mut move_command = Command::new("mv");
                move_command.args([
                    self.saved_filename.as_ref().unwrap().as_str(),
                    if is_audio_record {
                        video_filename.as_str()
                    } else {
                        self.saved_filename.as_ref().unwrap()
                    },
                ]);
                move_command.output().unwrap();
            }

            // if audio record, then merge video and audio
            if is_audio_record {
                Command::new("ffmpeg")
                    .args([
                        "-i",
                        video_filename.as_str(),
                        "-f",
                        "ogg",
                        "-i",
                        audio_filename.as_str(),
                        "-crf",
                        "23", // default quality
                        "-c:a",
                        "aac",
                        self.saved_filename.as_ref().unwrap(),
                        "-y",
                    ])
                    .output()
                    .expect("failed to merge video and audio");

                std::fs::remove_file(audio_filename).unwrap();
            }

            std::fs::remove_file(video_filename).unwrap();
        }
        // if only audio is recording then convert it to chosen format
        else if is_audio_record {
            Command::new("ffmpeg")
                .args([
                    "-f",
                    "ogg",
                    "-i",
                    audio_filename.as_str(),
                    self.saved_filename.as_ref().unwrap(),
                ])
                .output()
                .expect("failed convert audio to video");

            std::fs::remove_file(audio_filename).unwrap();
        }

        // execute command after finish recording
        if self.command.text().trim() != "" {
            Exec::shell(self.command.text().trim()).popen().unwrap();
        }
    }

    pub fn play_record(self) {
        if self.saved_filename.is_some() {
            if is_snap() {
                // open the video using snapctrl for snap package
                Command::new("snapctl")
                    .arg("user-open")
                    .arg(self.saved_filename.unwrap())
                    .spawn()
                    .unwrap();
            } else {
                Command::new("xdg-open")
                    .arg(self.saved_filename.unwrap())
                    .spawn()
                    .unwrap();
            }
        }
    }
}
