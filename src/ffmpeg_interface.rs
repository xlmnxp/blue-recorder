extern crate subprocess;

use chrono::prelude::*;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::{CheckButton, ComboBoxText, Entry, FileChooserNative, ProgressBar, SpinButton, Window};
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType, ResponseType};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use subprocess::Exec;

#[derive(Clone)]
pub struct ProgressWidget {
    pub progress_dialog: MessageDialog,
    pub progressbar: ProgressBar,
}

impl ProgressWidget {
    pub fn new(
        progress_dialog: MessageDialog,
        progressbar: ProgressBar,
    ) -> ProgressWidget {
        ProgressWidget {
            progress_dialog,
            progressbar,
        }
    }

    pub fn set_progress(&self, title: String, value: i32, max: i32) {
        let progress_precentage: f64 = value as f64 / max as f64;
        self.progressbar.set_text(Some(&title));
        self.progressbar.set_fraction(progress_precentage);
    }

    pub fn show(&self) {
        self.progressbar.set_fraction(0.0);
        self.progress_dialog.show();
    }

    pub fn hide(&self) {
        self.progress_dialog.hide();
    }
}

#[derive(Clone)]
pub struct Ffmpeg {
    pub filename: (FileChooserNative, Entry, ComboBoxText),
    pub record_video: CheckButton,
    pub record_audio: CheckButton,
    pub audio_id: ComboBoxText,
    pub record_mouse: CheckButton,
    pub follow_mouse: CheckButton,
    pub record_frames: SpinButton,
    pub command: Entry,
    pub video_process_id: Option<u32>,
    pub audio_process_id: Option<u32>,
    pub saved_filename: Option<String>,
    pub unbound: Option<Sender<bool>>,
    pub progress_widget: ProgressWidget,
    pub window: Window,
    pub overwrite: CheckButton,
}

impl Ffmpeg {
    pub fn start_record(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> (Option<u32>, Option<u32>) {
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

        if !self.overwrite.is_active() && is_file_already_exists {
            let message_dialog = MessageDialog::new(
                Some(&self.window),
                DialogFlags::empty(),
                MessageType::Warning,
                ButtonsType::Ok,
                &gettext("File already exist."),
            );
            message_dialog.show();
            message_dialog.connect_response(
                glib::clone!(@strong message_dialog => move |_, response| {
                    if response == ResponseType::Ok {
                        message_dialog.hide();
                    }
                    message_dialog.hide();
                }),
            );
            return (None, None);
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
            self.audio_process_id = Some(ffmpeg_command.spawn().unwrap().id());
        }

        if self.record_video.is_active() {
            let mut ffmpeg_command: Command = Command::new("ffmpeg");

            // Record video with specified width and hight
            ffmpeg_command.arg("-video_size");
            ffmpeg_command.arg(format!("{}x{}", width, height));
            ffmpeg_command.arg("-framerate");
            ffmpeg_command.arg(format!("{}", self.record_frames.value()));
            ffmpeg_command.arg("-f");
            ffmpeg_command.arg("x11grab");
            ffmpeg_command.arg("-i");
            ffmpeg_command.arg(format!(
                "{}+{},{}",
                std::env::var("DISPLAY")
                    .unwrap_or_else(|_| ":0".to_string())
                    .as_str(),
                x,
                y
            ));

            // If show mouse switch is enabled, draw the mouse to video
            ffmpeg_command.arg("-draw_mouse");
            if self.record_mouse.is_active() {
                ffmpeg_command.arg("1");
            } else {
                ffmpeg_command.arg("0");
            }

            // If follow mouse switch is enabled, follow the mouse
            if self.follow_mouse.is_active() {
                ffmpeg_command.arg("-follow_mouse");
                ffmpeg_command.arg("centered");
            }
            ffmpeg_command.arg("-crf");
            ffmpeg_command.arg("1");
            ffmpeg_command.arg(self.saved_filename.as_ref().unwrap());
            ffmpeg_command.arg("-y");
            // Start recording and return the process id
            self.video_process_id = Some(ffmpeg_command.spawn().unwrap().id());
            return (self.video_process_id, self.audio_process_id);
        }

        (None, None)
    }

    pub fn stop_record(&self) {
        self.progress_widget.show();
        // Kill the process to stop recording
        self.progress_widget.set_progress("".to_string(), 1, 6);

        if self.video_process_id.is_some() {
            self.progress_widget
                .set_progress("Stop Recording Video".to_string(), 1, 6);
            Command::new("kill")
                .arg(format!("{}", self.video_process_id.unwrap()))
                .output()
                .unwrap();
        }

        self.progress_widget.set_progress("".to_string(), 2, 6);

        if self.audio_process_id.is_some() {
            self.progress_widget
                .set_progress("Stop Recording Audio".to_string(), 2, 6);
            Command::new("kill")
                .arg(format!("{}", self.audio_process_id.unwrap()))
                .output()
                .unwrap();
        }

        let is_video_record = std::path::Path::new(
            format!(
                "{}{}",
                self.saved_filename.as_ref().unwrap_or(&String::from("")),
                { "" }
            )
            .as_str(),
        )
        .exists();
        let is_audio_record = std::path::Path::new(
            format!(
                "{}.temp.audio",
                self.saved_filename.as_ref().unwrap_or(&String::from(""))
            )
            .as_str(),
        )
        .exists();

        if is_video_record {
            let mut move_command = Command::new("mv");
            move_command.arg(format!("{}{}", self.saved_filename.as_ref().unwrap(), {
                ""
            }));
            move_command.arg(format!(
                "{}{}",
                self.saved_filename.as_ref().unwrap_or(&String::new()),
                if is_audio_record {
                    format!(
                        ".temp.without.audio.{}",
                        self.filename.2.active_id().unwrap()
                    )
                } else {
                    "".to_string()
                }
            ));
            move_command.output().unwrap();

            self.progress_widget.set_progress("".to_string(), 4, 6);

            // If audio record, then merge video with audio
            if is_audio_record {
                self.progress_widget
                    .set_progress("Save Audio Recording".to_string(), 4, 6);
                let mut ffmpeg_audio_merge_command = Command::new("ffmpeg");
                ffmpeg_audio_merge_command.arg("-i");
                ffmpeg_audio_merge_command.arg(format!(
                    "{}.temp.without.audio.{}",
                    self.saved_filename.as_ref().unwrap(),
                    self.filename.2.active_id().unwrap()
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
                sleep(Duration::from_secs(1));
                ffmpeg_audio_merge_command.output().unwrap();
                std::fs::remove_file(format!(
                    "{}.temp.audio",
                    self.saved_filename.as_ref().unwrap()
                ))
                .unwrap();
                std::fs::remove_file(format!(
                    "{}.temp.without.audio.{}",
                    self.saved_filename.as_ref().unwrap(),
                    self.filename.2.active_id().unwrap()
                ))
                .unwrap();
            }
        }

        self.progress_widget.set_progress("".to_string(), 5, 6);

        // Execute command after finish recording
        if self.command.text().trim() != "" {
            self.progress_widget.set_progress(
                "execute custom command after finish".to_string(),
                5,
                6,
            );
            Exec::shell(self.command.text().trim()).popen().unwrap();
        }

        self.progress_widget
            .set_progress("Finish".to_string(), 6, 6);
        self.progress_widget.hide();
    }

    pub fn play_record(self) {
        if self.saved_filename.is_some() {
            if is_snap() {
                // Open the video using snapctrl for snap package
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

fn is_snap() -> bool {
    !std::env::var("SNAP").unwrap_or_default().is_empty()
}
