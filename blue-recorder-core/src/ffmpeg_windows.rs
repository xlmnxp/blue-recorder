use anyhow::{anyhow, Result};
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use tempfile;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone)]
pub struct Ffmpeg {
    pub filename: String,
    pub output: String,
    pub temp_video_filename: String,
    pub window_title: String,
    pub command: Option<String>,
    pub audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_record_bitrate: u16,
    pub record_delay: u16,
    pub record_frames: u16,
    pub video_record_bitrate: u16,
    pub record_audio: bool,
    pub record_mouse: bool,
    pub record_video: bool,
}

impl Ffmpeg {
    // Start video recording
    pub fn start_video(&mut self, width: u16, height: u16, x: u16, y: u16,  mode: RecordMode) -> Result<()> {
        let display = match mode {
            RecordMode::Area => "desktop",
            RecordMode::Screen => "desktop",
            RecordMode::Window => &format!("title={}", &self.window_title),
        };
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "gdigrab";
        if self.output == "gif" {
            self.output = String::from("mp4");
        }

        // Record video to tmp if audio record enabled
        if self.record_audio {
            let video_tempfile = tempfile::Builder::new().suffix(&format!(".{}", &self.output))
                                                         .tempfile()?
                                                         .keep()?;
            self.temp_video_filename = Path::new(&video_tempfile.1).file_name()
                                                                   .ok_or_else(|| anyhow!("cannot get video temporary file name"))?
                                                                   .to_string_lossy()
                                                                   .to_string();
        }
        // Video format
        ffmpeg_command.format(format);

        // if show mouse switch is enabled, draw the mouse to video
        if self.record_mouse {
            ffmpeg_command.args(["-draw_mouse", "1"]);
        } else {
            ffmpeg_command.args(["-draw_mouse", "0"]);
        };

        // Disable frame rate if value is zero
        if self.record_frames > 0 {
            ffmpeg_command.args(["-framerate", &self.record_frames.to_string()]);
        }

        // Record video with specified width and hight
        ffmpeg_command.size(width.into(), height.into()).args([
            "-offset_x", &x.to_string(),
            "-offset_y", &y.to_string()
        ]);

        // input
        ffmpeg_command.input(display);

        // Disable bitrate if value is zero
        if self.video_record_bitrate > 0 {
            ffmpeg_command.args([
                "-b:v",
                &format!("{}K", self.video_record_bitrate),
            ]);
        }

        // tmp file
        if !self.record_audio {
            ffmpeg_command.args(["-hls_flags", "temp_file"]);
        }

        // Output
        ffmpeg_command.args([
            {
                if self.record_audio {
                    &self.temp_video_filename
                } else {
                    &self.filename
                }
            },
        ]);
        ffmpeg_command.overwrite();

        // Sleep for delay
        sleep(Duration::from_secs(self.record_delay as u64));

        // Start recording and return the process id
        self.video_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn()?)));

        Ok(())
    }

    pub fn stop_video(&mut self) -> Result<()> {
        // kill the process to stop recording
        if self.video_process.is_some() {
            self.video_process
                .clone()
                .ok_or_else(|| anyhow!("not exiting the video recording process successfully"))?
                .borrow_mut()
                .quit()?;
        }
        Ok(())
    }
}
