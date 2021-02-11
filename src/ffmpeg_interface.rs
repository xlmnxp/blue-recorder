use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use gtk::{CheckButton, SpinButton, ComboBoxText, FileChooser, Entry, ToggleButtonExt, SpinButtonExt, ComboBoxExt, FileChooserExt, EntryExt};

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
    pub process_id: Option<u32>
}

impl Ffmpeg {
    pub fn start_record(&mut self, x: i16, y: i16, width: i16, height: i16) -> u32 {
        if self.process_id.is_some() {
            Command::new("kill").arg(format!("{}", self.process_id.unwrap())).output().unwrap();
        }

        let mut ffmpeg_command: Command = Command::new("ffmpeg");

        // if recorder video switch is enabled, record video with specified width and hight
        if self.record_video.get_active() {
            ffmpeg_command.arg("-video_size");
            ffmpeg_command.arg(format!("{}x{}", width, height));
        }

        // if show mouse switch is enabled, draw the mouse to video
        if self.record_mouse.get_active() {
            ffmpeg_command.arg("-draw_mouse");
            ffmpeg_command.arg("1");
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


        ffmpeg_command.arg({
            self.filename.0.get_filename()
            .unwrap()
            .join(PathBuf::from(format!(
                "{}.{}",
                if self.filename.1.get_text().to_string().trim().eq("") {
                    self.filename.1.get_text().to_string()
                } else {
                    self.filename.1.get_text().to_string().trim().to_string()
                },
                self.filename.2.get_active_id().unwrap().to_string()
            )))
        });
        ffmpeg_command.arg("-y");

        // sleep for delay
        sleep(Duration::from_secs(self.record_delay.get_value() as u64));

        // start recording and return the process id
        self.process_id = Some(ffmpeg_command.spawn().unwrap().id());
        println!("{}", self.process_id.unwrap());  
        self.process_id.unwrap()
    }

    pub fn stop_record(self) {
        if self.process_id.is_some() {
            Command::new("kill").arg(format!("{}", self.process_id.unwrap())).output().unwrap();
        }
    }

    pub fn play_record(self) {
        Command::new("xdg-open").arg({
            self.filename.0.get_filename()
            .unwrap()
            .join(PathBuf::from(format!(
                "{}.{}",
                if self.filename.1.get_text().to_string().trim().eq("") {
                    self.filename.1.get_text().to_string()
                } else {
                    self.filename.1.get_text().to_string().trim().to_string()
                },
                self.filename.2.get_active_id().unwrap().to_string()
            )))
        }).output().unwrap();
    }
}
