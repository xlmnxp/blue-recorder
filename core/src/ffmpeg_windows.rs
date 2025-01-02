use anyhow::{anyhow, Error, Result};
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use tempfile;
use std::{cell::RefCell, time::Instant};
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::utils::{is_input_audio_record, is_output_audio_record, is_valide, is_video_record, RecordMode};

#[derive(Clone)]
pub struct Ffmpeg {
    pub audio_input_id: String,
    pub audio_output_id: String,
    pub filename: String,
    pub output: String,
    pub temp_input_audio_filename: String,
    pub temp_output_audio_filename: String,
    pub temp_video_filename: String,
    pub window_title: String,
    pub height: Option<u16>,
    pub input_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub output_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_record_bitrate: u16,
    pub record_delay: u16,
    pub record_frames: u16,
    pub video_record_bitrate: u16,
    pub follow_mouse: bool,
    pub record_mouse: bool,
    pub show_area: bool,
}

impl Ffmpeg {
    // Start video recording
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16,  mode: RecordMode) -> Result<()> {
        let display = match mode {
            RecordMode::Area => "desktop",
            RecordMode::Screen => "desktop",
            RecordMode::Window => &format!("title={}", &self.window_title),
        };
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "gdigrab";

        // Record video to tmp if audio record enabled
        if !self.audio_input_id.is_empty()
            || !self.audio_output_id.is_empty()
            || self.output == "gif"
        {
            let suffix = if self.output == "gif" {
                    ".mp4"
            } else {
                &format!(".{}", &self.output)
            };
            let video_tempfile = tempfile::Builder::new().prefix("ffmpeg-video-")
                                                         .suffix(suffix)
                                                         .tempfile()?
                                                         .keep()?;
            self.temp_video_filename = Path::new(&video_tempfile.1).to_string_lossy()
                                                                   .to_string();
        }
        // Video format
        ffmpeg_command.format(format);

        // Show grabbed area
        if self.show_area {
            ffmpeg_command.args(["-show_region", "1"]);
        }
       
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
        if let RecordMode::Area = mode  {
            ffmpeg_command.size(width.into(), height.into()).args([
                "-offset_x", &x.to_string(),
                "-offset_y", &y.to_string()
            ]);
        }

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
        if self.audio_input_id.is_empty() &&
            self.audio_output_id.is_empty() &&
            self.output != "gif"
        {
            ffmpeg_command.args(["-hls_flags", "temp_file"]);
        }

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        ffmpeg_command.args([
            {
                if !self.audio_input_id.is_empty()
                    || !self.audio_output_id.is_empty()
                    || self.output == "gif"
                {
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

    // Stop video recording
    pub fn stop_video(&mut self) -> Result<()> {
        // Quit the process to stop recording
        if self.video_process.is_some() {
            self.video_process
                .clone()
                .ok_or_else(|| anyhow!("Not exiting the video recording process successfully."))?
                .borrow_mut()
                .quit()?;
        }
        Ok(())
    }

    // Start audio input recording
    pub fn start_input_audio(&mut self) -> Result<()> {
        let input_audio_tempfile = tempfile::Builder::new().prefix("ffmpeg-audio-")
                                                           .suffix(".ogg")
                                                           .tempfile()?
                                                           .keep()?;
        self.temp_input_audio_filename = Path::new(&input_audio_tempfile.1).to_string_lossy()
                                                                           .to_string();
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("dshow")
                      .input(format!("audio={}", &self.audio_input_id))
                      .format("ogg");
        // Disable bitrate if value is zero
        if self.audio_record_bitrate > 0 {
            ffmpeg_command.args([
                "-b:a",
                &format!("{}K", self.audio_record_bitrate),
            ]);
        }
        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);
        ffmpeg_command.arg(&self.temp_input_audio_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !is_video_record(&self.temp_video_filename) {
            sleep(Duration::from_secs(self.record_delay as u64));
        }

        // Start recording and return the process id
        self.input_audio_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn()?)));
        Ok(())
    }

    // Stop audio input recording
    pub fn stop_input_audio(&mut self) -> Result<()> {
        // Quit the process to stop recording
        if self.input_audio_process.is_some() {
            self.input_audio_process
                .clone()
                .ok_or_else(|| anyhow!("Not exiting the input audio recording process successfully."))?
                .borrow_mut()
                .quit()?;
      }
        Ok(())
    }

    // Start audio output recording
    pub fn start_output_audio(&mut self) -> Result<()> {
        let output_audio_tempfile = tempfile::Builder::new().prefix("ffmpeg-audio-")
                                                            .suffix(".ogg")
                                                            .tempfile()?
                                                            .keep()?;
        self.temp_output_audio_filename = Path::new(&output_audio_tempfile.1).to_string_lossy()
                                                                             .to_string();
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("dshow")
                      .input(format!("audio={}", &self.audio_input_id))
                      .format("ogg");
        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);
        ffmpeg_command.arg(&self.temp_output_audio_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !is_video_record(&self.temp_video_filename) && !is_input_audio_record(&self.temp_input_audio_filename) {
            sleep(Duration::from_secs(self.record_delay as u64));
        }

        // Start recording and return the process id
        self.output_audio_process = Some(Rc::new(RefCell::new(ffmpeg_command.spawn()?)));
        Ok(())
    }

    // Stop audio output recording
    pub fn stop_output_audio(&mut self) -> Result<()> {
        // Quit the process to stop recording
        if self.output_audio_process.is_some() {
            self.output_audio_process
                .clone()
                .ok_or_else(|| anyhow!("Not exiting the output audio recording process successfully."))?
                .borrow_mut()
                .quit()?;
        }
        Ok(())
    }

    // Merge tmp to target format
    pub fn merge(&mut self) -> Result<()> {
        if is_video_record(&self.temp_video_filename) {
            if self.output != "gif" {
                // Validate video file integrity
                let start_time = Instant::now();
                let duration = Duration::from_secs(60);
                loop {
                    if is_valide(&self.temp_video_filename)? {
                        break;
                    } else if Instant::now().duration_since(start_time) >= duration {
                        return Err(Error::msg("Unable to validate tmp video file."));
                    }
                }
                let mut ffmpeg_command = FfmpegCommand::new();
                ffmpeg_command.input(&self.temp_video_filename);
                ffmpeg_command.format("ogg");
                if is_input_audio_record(&self.temp_input_audio_filename) {
                    ffmpeg_command.input(&self.temp_input_audio_filename);
                }
                if is_output_audio_record(&self.temp_output_audio_filename) {
                    ffmpeg_command.input(&self.temp_output_audio_filename);
                }
                ffmpeg_command.args([
                    "-c:a",
                    "aac",
                    &self.filename,
                ]);
                ffmpeg_command.overwrite()
                  .spawn()?
                  .wait()?;
            } else {
                // Validate video file integrity
                let start_time = Instant::now();
                let duration = Duration::from_secs(60);
                loop {
                    if is_valide(&self.temp_video_filename)? {
                        break;
                    } else if Instant::now().duration_since(start_time) >= duration {
                        return Err(Error::msg("Unable to validate tmp video file."));
                    }
                }
                // Convert MP4 to GIF
                let filter = format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                                     self.record_frames,self.height.ok_or_else
                                     (|| anyhow!("Unable to get height value"))?);
                let ffmpeg_convert = format!("ffmpeg -i file:{} -filter_complex '{}' \
                                              -loop 0 {} -y", &self.temp_video_filename,filter,&self.filename);
                std::process::Command::new("sh").arg("-c").arg(&ffmpeg_convert).output()?;
            }
        } else if is_input_audio_record(&self.temp_input_audio_filename) {
            // Validate audio file integrity
            let start_time = Instant::now();
            let duration = Duration::from_secs(60);
            loop {
                if is_valide(&self.temp_input_audio_filename)? {
                    break;
                } else if Instant::now().duration_since(start_time) >= duration {
                    return Err(Error::msg("Unable to validate tmp video file."));
                }
            }
            // If only audio is recording then convert it to chosen format
            let mut ffmpeg_command = FfmpegCommand::new();
            ffmpeg_command.format("ogg");
            ffmpeg_command.input(&self.temp_input_audio_filename);
            if is_output_audio_record(&self.temp_output_audio_filename) {
                ffmpeg_command.input(&self.temp_output_audio_filename);
            }
            ffmpeg_command.args([
                "-c:a",
                "aac",
                &self.filename,
            ]).overwrite()
              .spawn()?
              .wait()?;
        } else {
            // Validate audio file integrity
            let start_time = Instant::now();
            let duration = Duration::from_secs(60);
            loop {
                if is_valide(&self.temp_output_audio_filename)? {
                    break;
                } else if Instant::now().duration_since(start_time) >= duration {
                    return Err(Error::msg("Unable to validate tmp video file."));
                }
            }
            // If only output audio is recording then convert it to chosen format
            let mut ffmpeg_command = FfmpegCommand::new();
            ffmpeg_command.format("ogg");
            ffmpeg_command.input(&self.temp_output_audio_filename);
            ffmpeg_command.arg(&self.filename)
                          .overwrite()
                          .spawn()?
                          .wait()?;
        }
        Ok(())
    }

    // Clean tmp
    pub fn clean(&mut self) -> Result<()> {
        let tmp_files = vec![ &self.temp_input_audio_filename, &self.temp_output_audio_filename, &self.temp_video_filename ];
        for file in tmp_files {
            if Path::new(file).try_exists()? {
                std::fs::remove_file(file)?;
            }
        }
        Ok(())
    }
}
