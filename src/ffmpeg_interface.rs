extern crate subprocess;
use crate::config_management;
use crate::utils::{is_snap, is_wayland};
use crate::wayland_record::{CursorModeTypes, RecordTypes, WaylandRecorder};
use chrono::prelude::*;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use gtk::{prelude::*, ResponseType};
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
use gtk::{CheckButton, ComboBoxText, Entry, FileChooserNative, SpinButton, Window};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use subprocess::Exec;
use filename::Filename;

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
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub height: Option<u16>,
    pub saved_filename: Option<String>,
    pub unbound: Option<Sender<bool>>,
    pub window: Window,
    pub record_wayland: WaylandRecorder,
    pub record_window: Rc<RefCell<bool>>,
    pub main_context: gtk::glib::MainContext,
    pub temp_video_filename: String,
    pub bundle: String,
    pub video_record_bitrate: SpinButton,
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
                &self.bundle,
            );

            let answer = self.main_context.block_on(message_dialog.run_future());
            message_dialog.close();

            if answer != ResponseType::Yes {
                return None;
            }
        }

        if self.record_video.is_active() && !is_wayland() && self.filename.2.active_id().unwrap().as_str() != "gif" {
            let mode = config_management::get("default", "mode");
            let format = "x11grab";
            let display = format!("{}+{},{}",
                                  std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                                  .as_str(),
                                  x,
                                  y
            );
            let mut ffmpeg_command = FfmpegCommand::new();

            // record video with specified width and hight
            if self.follow_mouse.is_active() && mode.as_str() == "screen" {
                let width = width as f32 * 0.95;
                let height = height as f32 * 0.95;
                ffmpeg_command.size(width as u32, height as u32);
            } else {
                ffmpeg_command.size(width.into(), height.into());
            }

            // if show mouse switch is enabled, draw the mouse to video
            if self.record_mouse.is_active() {
                ffmpeg_command.args(["-draw_mouse", "1"]);
            } else {
                ffmpeg_command.args(["-draw_mouse", "0"]);
            };

            // if follow mouse switch is enabled, follow the mouse
            if self.follow_mouse.is_active() {
                ffmpeg_command.args(["-follow_mouse", "centered"]);
            }

            // Disable frame rate if value is zero
            if self.record_frames.value() > 0.0 {
                ffmpeg_command.args(["-framerate", &self.record_frames.value().to_string()]);
            }

            // Video format && input
            ffmpeg_command.format(format)
                          .input(display);

            // Disable bitrate if value is zero
            if self.video_record_bitrate.value() > 0.0 {
                ffmpeg_command.args([
                    "-b:v",
                    &format!("{}K", self.video_record_bitrate.value()),
                ]);
            }

            let video_filename = format!(
                "{}.temp.without.audio.{}",
                self.saved_filename.as_ref().unwrap(),
                self.filename.2.active_id().unwrap()
            );

            // Output
            ffmpeg_command.args([
                {
                    if self.record_audio.is_active() {
                        video_filename.as_str()
                    } else {
                        self.saved_filename.as_ref().unwrap()
                    }
                },
            ]);
            ffmpeg_command.overwrite();

            // sleep for delay
            sleep(Duration::from_secs(self.record_delay.value() as u64));

            // start recording and return the process id
            self.video_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn().unwrap())));
        } else if self.record_video.is_active() && !is_wayland() && self.filename.2.active_id().unwrap().as_str() == "gif" {
            let mode = config_management::get("default", "mode");
            let format = "x11grab";
            let display = format!("{}+{},{}",
                                  std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                                  .as_str(),
                                  x,
                                  y
            );
            let mut ffmpeg_command = FfmpegCommand::new();

            // record video with specified width and hight
            if self.follow_mouse.is_active() && mode.as_str() == "screen" {
                let width = width as f32 * 0.95;
                let height = height as f32 * 0.95;
                ffmpeg_command.size(width as u32, height as u32);
            } else {
                ffmpeg_command.size(width.into(), height.into());
            }

            // if show mouse switch is enabled, draw the mouse to video
            if self.record_mouse.is_active() {
                ffmpeg_command.args(["-draw_mouse", "1"]);
            } else {
                ffmpeg_command.args(["-draw_mouse", "0"]);
            };

            // if follow mouse switch is enabled, follow the mouse
            if self.follow_mouse.is_active() {
                ffmpeg_command.args(["-follow_mouse", "centered"]);
            }

            // Disable frame rate if value is zero
            if self.record_frames.value() > 0.0 {
                ffmpeg_command.args(["-framerate", &self.record_frames.value().to_string()]);
            }

            // Video format && input
            ffmpeg_command.format(format)
                          .input(display);

            // Disable bitrate if value is zero
            if self.video_record_bitrate.value() > 0.0 {
                ffmpeg_command.args([
                    "-b:v",
                    &format!("{}K", self.video_record_bitrate.value()),
                ]);
            }

            let video_filename = format!(
                "{}.temp.without.audio.{}",
                self.saved_filename.as_ref().unwrap(),
                self.filename.2.active_id().unwrap()
            ).replace("gif", "mp4");

            // Output
            ffmpeg_command.arg(video_filename.as_str())
                          .overwrite();

            // sleep for delay
            sleep(Duration::from_secs(self.record_delay.value() as u64));

            // start recording and return the process id
            self.video_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn().unwrap())));
            self.height = Some(height);
        } else if self.record_video.is_active() && is_wayland() {
            sleep(Duration::from_secs(self.record_delay.value() as u64));

            let tempfile = tempfile::NamedTempFile::new().expect("cannot create temp file").keep().expect("cannot keep temp file");
            self.temp_video_filename = tempfile.0.file_name().expect("cannot get file name").to_str().unwrap().to_string();

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
            let mut ffmpeg_command = FfmpegCommand::new();
            ffmpeg_command.format("pulse")
                          .input(&self.audio_id.active_id().unwrap())
                          .format("ogg");
            ffmpeg_command.arg(format!(
                "{}.temp.audio",
                self.saved_filename.as_ref().unwrap()
            ));
            ffmpeg_command.overwrite();
            self.audio_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn().unwrap())));
        }

        Some(())
    }

    pub fn stop_record(&mut self) {
        // kill the process to stop recording
        if self.video_process.is_some() {
            self.video_process
                .clone()
                .unwrap()
                .borrow_mut()
                .quit()
                .unwrap();

            println!("video killed");
        } else if is_wayland() {
            self.main_context.block_on(self.record_wayland.stop());
        }

        if self.audio_process.is_some() {
            self.audio_process
                .clone()
                .unwrap()
                .borrow_mut()
                .quit()
                .unwrap();

            println!("audio killed");
        }

        let video_filename = {
            if is_wayland() {
                self.temp_video_filename.clone()
            } else if !is_wayland() && self.filename.2.active_id().unwrap().as_str() == "gif" {
                format!(
                    "{}.temp.without.audio.{}",
                    self.saved_filename.as_ref().unwrap(),
                    self.filename.2.active_id().unwrap()
                ).replace("gif", "mp4")
            } else {
                format!(
                    "{}.temp.without.audio.{}",
                    self.saved_filename.as_ref().unwrap(),
                    self.filename.2.active_id().unwrap()
                )
            }
        };

        let audio_filename = format!("{}.temp.audio", self.saved_filename.as_ref().unwrap());

        let is_video_record = {
            std::path::Path::new(video_filename.as_str()).exists()
        };
        let is_audio_record = std::path::Path::new(audio_filename.as_str()).exists();

        if is_video_record {
            if is_wayland() {
                // convert webm to specified format
                let mut ffmpeg_command = FfmpegCommand::new();
                ffmpeg_command.input(self.temp_video_filename.as_str());
                if self.video_record_bitrate.value() > 0.0 {
                    ffmpeg_command.args([
                        "-b:v",
                        &format!("{}K", self.video_record_bitrate.value()),
                    ]);
                }
                ffmpeg_command.args([
                    "-c:a",
                    self.filename.2.active_id().unwrap().as_str(),
                    self.saved_filename.as_ref().unwrap(),
                ]).overwrite()
                  .spawn()
                  .unwrap().wait().unwrap();
            } else if !is_wayland() && self.filename.2.active_id().unwrap().as_str() == "gif" {
                let fps = 100/self.record_frames.value_as_int();
                let scale = self.height.unwrap();
                Command::new("ffmpeg").arg("-i")
                                      .arg(format!("file:{}", video_filename.as_str()))
                                      .arg("-filter_complex")
                                      .arg(format!("fps={},scale={}:-1:flags=lanczos,[0]split[s0][s1]; [s0]palettegen[p]; [s1][p]paletteuse",
                                                   fps,scale))
                                      .args(["-loop", "0"])
                                      .arg(self.saved_filename.as_ref().unwrap())
                                      .status()
                                      .unwrap();
                //let mut ffmpeg_command = FfmpegCommand::new();
                /*ffmpeg_command.input(format!("file:{}", video_filename.as_str()))
                              .filter_complex(
                                  format!("fps={},scale={}:-1:flags=lanczos,[0]split[s0][s1]; [s0]palettegen[p]; [s1][p]paletteuse",
                                  fps,scale)
                              )
                              .args(["-loop", "0"])
                              .output(self.saved_filename.as_ref().unwrap())
                              .overwrite().spawn().unwrap().wait().expect("failed to convert video to gif");*/
                if is_audio_record {
                    std::fs::remove_file(audio_filename.clone()).unwrap();
                }
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
            if is_audio_record && self.filename.2.active_id().unwrap().as_str() != "gif" {
                FfmpegCommand::new().input(video_filename.as_str())
                                    .format("ogg")
                                    .input(audio_filename.as_str())
                                    .args([
                                        "-c:a",
                                        "aac",
                                        self.saved_filename.as_ref().unwrap(),
                                    ])
                                    .overwrite()
                                    .spawn()
                                    .unwrap()
                                    .wait()
                                    .expect("failed to merge video and audio");

                std::fs::remove_file(audio_filename).unwrap();
            }

            std::fs::remove_file(video_filename).unwrap();
        }
        // if only audio is recording then convert it to chosen format
        else if is_audio_record {
            let mut ffmpeg_command = FfmpegCommand::new();
            ffmpeg_command.format("ogg").input(audio_filename.as_str()).arg(
                self.saved_filename.as_ref().unwrap(),
            ).spawn()
             .unwrap()
             .wait()
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
                open::that(self.saved_filename.unwrap()).unwrap();
            }
        }
    }
}
