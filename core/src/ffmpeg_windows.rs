use anyhow::{anyhow, Error, Result};
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use tempfile;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use crate::utils::{is_valid, is_video_record, RecordMode};

#[derive(Clone)]
pub struct Ffmpeg {
    pub audio_input_id: String,
    pub audio_output_id: String,
    pub filename: String,
    pub output: String,
    pub temp_video_filename: String,
    pub saved_filename: String,
    pub height: Option<u16>,
    pub input_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub output_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_record_bitrate: u16,
    pub record_delay: u16,
    pub record_frames: u16,
    pub video_record_bitrate: u16,
    pub audio_input_enabled: bool,
    pub audio_output_enabled: bool,
    pub follow_mouse: bool,
    pub record_mouse: bool,
    pub show_area: bool,
    pub video_enabled: bool,
}

impl Ffmpeg {
    // Start video recording
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16,  mode: RecordMode, title: String) -> Result<()> {
        let display = match mode {
            RecordMode::Area => "desktop",
            RecordMode::Screen => "desktop",
            RecordMode::Window => &format!("title={}", title),
        };
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "gdigrab";
        self.output = Path::new(&self.filename)
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Record video to tmp if output is GIF
        if self.output == "gif" {
            let suffix =  ".mp4";
            let video_tempfile = tempfile::Builder::new().prefix(".ffmpeg-video-")
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

        // Record audio input
        if self.audio_input_enabled {
            ffmpeg_command.format("dshow");
            ffmpeg_command.input(format!("audio={}", &self.audio_input_id));
        }

        // Record audio output
        if self.audio_output_enabled {
            ffmpeg_command.format("dshow");
            ffmpeg_command.input(format!("audio={}", &self.audio_output_id));
        }

        // Disable bitrate if value is zero
        if self.video_record_bitrate > 0 {
            ffmpeg_command.args([
                "-b:v",
                &format!("{}K", self.video_record_bitrate.to_string()),
            ]);
        }

        // Disable audio bitrate if value is zero
        if self.audio_input_enabled || self.audio_output_enabled {
            if self.audio_record_bitrate > 0 {
                ffmpeg_command.args([
                    "-b:a",
                    &format!("{}K", self.audio_record_bitrate),
                ]);
            }
        }

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        let saved_filename = self.saved_filename.clone();
        ffmpeg_command.args([
            {
                if self.output == "gif"
                {
                    &self.temp_video_filename
                } else {
                    &saved_filename
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
            match self.video_process
                      .clone()
                      .ok_or_else(|| anyhow!("Not exiting the video recording process successfully."))?
                      .borrow_mut()
                      .quit() {
                          Ok(_) => {
                              if self.output == "gif" {
                                  match self.merge() {
                                      Ok(_) => {
                                          self.clean()?;
                                      },
                                      Err(error) => {
                                          self.clean()?;
                                          return Err(Error::msg(format!("{}", error)));
                                      }
                                  }
                              }
                          },
                          Err(error) => {
                              if self.output == "gif" {
                                  self.clean()?;
                              } else {
                                  self.temp_video_filename = self.saved_filename.clone();
                                  self.clean()?;
                              }
                              return Err(Error::msg(format!("{}", error)));
                          },
                      }
        }
        Ok(())
    }

    // Start audio input recording
    pub fn start_input_audio(&mut self) -> Result<()> {
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("dshow")
                      .input(format!("audio={}", &self.audio_input_id));
        if self.audio_output_enabled {
            ffmpeg_command.format("dshow")
                          .input(format!("audio={}", &self.audio_output_id));
        }

        // Disable bitrate if value is zero
        if self.audio_record_bitrate > 0 {
            ffmpeg_command.args([
                "-b:a",
                &format!("{}K", self.audio_record_bitrate),
            ]);
        }

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        ffmpeg_command.arg(&self.saved_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !self.video_enabled {
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
            match self.input_audio_process
                      .clone()
                      .ok_or_else(|| anyhow!("Not exiting the input audio recording process successfully."))?
                      .borrow_mut()
                      .quit() {
                          Ok(_) => {
                              // Continue
                          },
                          Err(error) => {
                              self.temp_video_filename = self.saved_filename.clone();
                              self.clean()?;
                              return Err(Error::msg(format!("{}", error)));
                          },
                      }
        }
        Ok(())
    }

    // Start audio output recording
    pub fn start_output_audio(&mut self) -> Result<()> {
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("dshow")
                      .input(format!("audio={}", &self.audio_output_id));

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        ffmpeg_command.arg(&self.saved_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !self.video_enabled && !self.audio_input_enabled {
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
            match self.output_audio_process
                      .clone()
                      .ok_or_else(|| anyhow!("Not exiting the output audio recording process successfully."))?
                      .borrow_mut()
                      .quit() {
                          Ok(_) => {
                              // Continue
                            },
                          Err(error) => {
                              self.temp_video_filename = self.saved_filename.clone();
                              self.clean()?;
                              return Err(Error::msg(format!("{}", error)));
                          },
                      }
        }
        Ok(())
    }

    // Merge tmp to target format
    pub fn merge(&mut self) -> Result<()> {
        if is_video_record(&self.temp_video_filename) {
            // Validate video file integrity
            match is_valid(&self.temp_video_filename) {
                Ok(_) => {
                    // Convert MP4 to GIF
                    let filter = format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                                         self.record_frames,
                                         self.height.ok_or_else
                                         (|| anyhow!("Unable to get height value"))?);
                    let ffmpeg_convert = format!("ffmpeg -i file:{} -filter_complex '{}' \
                                                  -loop 0 {} -y", &self.temp_video_filename,filter,
                                                 &self.saved_filename
                                                 .clone());
                    match std::process::Command::new("sh").arg("-c").arg(&ffmpeg_convert).output() {
                        Ok(_) => {
                            // Do nothing
                        },
                        Err(error) => {
                            return Err(Error::msg(format!("{}", error)));
                        },
                    }
                },
                Err(error) => {
                    return Err(Error::msg(format!("{}", error)));
                },
            }
        }
        Ok(())
    }

    // Clean tmp
    pub fn clean(&mut self) -> Result<()> {
        if Path::new(&self.temp_video_filename).try_exists()? {
            std::fs::remove_file(&self.temp_video_filename)?;
        }
        Ok(())
    }

    // Kill process
    pub fn kill(&mut self) -> Result<()> {
        if self.video_process.is_some() {
            let pid = self.video_process
                          .clone()
                          .ok_or_else(|| anyhow!("Unable to kill the video recording process successfully."))?
                          .borrow_mut()
                          .as_inner().id();
            std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/F")
                .output()?;
        }
        if self.input_audio_process.is_some() {
            let pid = self.input_audio_process
                          .clone()
                          .ok_or_else(|| anyhow!("Unable to kill the input audio recording process successfully."))?
                          .borrow_mut()
                          .as_inner().id();
            std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/F")
                .output()?;
        }
        if self.output_audio_process.is_some() {
            let pid = self.output_audio_process
                          .clone()
                          .ok_or_else(|| anyhow!("Unable to kill the output audio recording process successfully."))?
                          .borrow_mut()
                          .as_inner().id();
            std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/F")
                .output()?;
        }
        Ok(())
    }
}
