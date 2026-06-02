#[cfg(feature = "gtk")]
use adw::gtk::{CheckButton, ComboBoxText, Entry, FileChooserNative, SpinButton};
#[cfg(feature = "gtk")]
use adw::gtk::prelude::*;
use anyhow::{anyhow, Error, Result};
#[cfg(feature = "gtk")]
use chrono::Utc;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use tempfile;
use std::cell::RefCell;
use std::path::Path;
#[cfg(feature = "gtk")]
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
#[cfg(feature = "cmd")]
use std::time::Instant;

use crate::utils::{is_video_record, is_wayland, RecordMode};
#[cfg(feature = "cmd")]
use crate::utils::{is_input_audio_record, is_output_audio_record, is_valid};
#[cfg(feature = "gtk")]
use crate::wayland_linux::{CursorModeTypes, RecordTypes, WaylandRecorder};

#[cfg(feature = "cmd")]
#[derive(Clone)]
pub struct Ffmpeg {
    pub audio_input_id: String,
    pub audio_output_id: String,
    pub filename: String,
    pub output: String,
    pub temp_input_audio_filename: String,
    pub temp_output_audio_filename: String,
    pub temp_video_filename: String,
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

#[cfg(feature = "gtk")]
#[derive(Clone)]
pub struct Ffmpeg {
    pub audio_input_id: ComboBoxText,
    pub audio_output_id: String,
    pub filename: (FileChooserNative, Entry, ComboBoxText),
    pub output: String,
    pub temp_video_filename: String,
    pub temp_input_audio_filename: String,
    pub temp_output_audio_filename: String,
    pub saved_filename: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub input_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub output_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_record_bitrate: SpinButton,
    pub record_delay: SpinButton,
    pub record_frames: SpinButton,
    pub video_record_bitrate: SpinButton,
    pub audio_input_switch: CheckButton,
    pub audio_output_switch: CheckButton,
    pub follow_mouse: CheckButton,
    pub record_mouse: CheckButton,
    pub show_area: CheckButton,
    pub video_switch: CheckButton,
    pub wayland_recorder: WaylandRecorder,
}

#[cfg(feature = "cmd")]
impl Ffmpeg {
    // Start video recording
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16,  mode: RecordMode) -> Result<()> {
        //if let RecordMode::Window == mode && !self.follow_mouse.is_active() {
            // TODO pulse = gstreamer for video  && add to cmd linux + add convert function to gstreamer ouput
        //} else {
            let display = format!("{}+{},{}",
                                  std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                                  .as_str(),
                                  x,
                                  y
            );
            let mut ffmpeg_command = FfmpegCommand::new();
            let format = "x11grab";
            self.width = Some(width);
            self.height = Some(height);

            // Record video to tmp if output is GIF
            if !self.audio_input_id.is_empty()
                || !self.audio_output_id.is_empty()
                || self.output == "gif"
            {
                let suffix = if self.output == "gif" {
                    ".mp4"
                } else {
                    &format!(".{}", &self.output)
                };
                let video_tempfile = tempfile::Builder::new().prefix(".ffmpeg-video-")
                                                             .suffix(suffix)
                                                             .tempfile()?
                                                             .keep()?;
                self.temp_video_filename = Path::new(&video_tempfile.1).to_string_lossy()
                                                                       .to_string();
            }

            // Record video with specified width and hight
            if self.follow_mouse {
                match mode {
                    RecordMode::Screen => {
                        let width = width as f32 * 0.95;
                        let height = height as f32 * 0.95;
                        ffmpeg_command.size(width as u32, height as u32);
                    },
                    _=> {
                        ffmpeg_command.size(width as u32, height as u32);
                    }
                }
            } else {
                ffmpeg_command.size(width as u32, height as u32);
            }

            // Show grabbed area
            if self.show_area {
                ffmpeg_command.args(["-show_region", "1"]);
            }

            // If show mouse switch is enabled, draw the mouse to video
            if self.record_mouse {
                ffmpeg_command.args(["-draw_mouse", "1"]);
            } else {
                ffmpeg_command.args(["-draw_mouse", "0"]);
            };

            // Follow the mouse
            if self.follow_mouse {
                ffmpeg_command.args(["-follow_mouse", "centered"]);
            }

            // Disable frame rate if value is zero
            if self.record_frames > 0 {
                ffmpeg_command.args(["-framerate", &self.record_frames.to_string()]);
            }

            // Video format && input
            ffmpeg_command.format(format)
                          .input(display);

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
        let input_audio_tempfile = tempfile::Builder::new().prefix(".ffmpeg-audio-")
                                                           .suffix(".ogg")
                                                           .tempfile()?
                                                           .keep()?;
        self.temp_input_audio_filename = Path::new(&input_audio_tempfile.1).to_string_lossy()
                                                                           .to_string();
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("pulse")
                      .input(&self.audio_input_id)
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
        let output_audio_tempfile = tempfile::Builder::new().prefix(".ffmpeg-audio-")
                                                            .suffix(".ogg")
                                                            .tempfile()?
                                                            .keep()?;
        self.temp_output_audio_filename = Path::new(&output_audio_tempfile.1).to_string_lossy()
                                                                             .to_string();
        let mut ffmpeg_command = FfmpegCommand::new("ffmpeg");
        ffmpeg_command.format("pulse")
                      .input(&self.audio_output_id)
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
                let duration = Duration::from_secs(300);
                loop {
                    if is_valid(&self.temp_video_filename)? {
                        break;
                    } else if Instant::now().duration_since(start_time) >= duration {
                        return Err(Error::msg("Unable to validate tmp video file."));
                    }
                }
                if is_input_audio_record(&self.temp_input_audio_filename) ||
                    is_output_audio_record(&self.temp_output_audio_filename) {
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
                            &self.saved_filename.clone()
                        ]);
                        ffmpeg_command.overwrite()
                                      .spawn()?
                                      .wait()?;
                    } else {
                        std::fs::copy(&self.temp_video_filename, &self.saved_filename)?;
                    }
            } else {
                // Validate video file integrity
                let start_time = Instant::now();
                let duration = Duration::from_secs(300);
                loop {
                    if is_valid(&self.temp_video_filename)? {
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
                match std::process::Command::new("sh").arg("-c").arg(&ffmpeg_convert).status() {
                    Ok(_) => {
                        // Do nothing
                        },
                    Err(error) => {
                        return Err(Error::msg(format!("{}", error)));
                    },
                }
            }
        } else if is_input_audio_record(&self.temp_input_audio_filename) {
            // Validate audio file integrity
            let start_time = Instant::now();
            let duration = Duration::from_secs(300);
            loop {
                if is_valid(&self.temp_input_audio_filename)? {
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
            let duration = Duration::from_secs(300);
            loop {
                if is_valid(&self.temp_output_audio_filename)? {
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

    // Kill process
    pub fn kill(&mut self) -> Result<()> {
        if self.video_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.video_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the video recording process successfully."))?
                        .borrow_mut()
                        .as_inner().id()
                )).output()?;
        }
        if self.input_audio_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.input_audio_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the intput audio recording process successfully."))?
                        .borrow_mut()
                        .as_inner().id()
                )).output()?;
        }
        if self.output_audio_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.output_audio_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the output audio recording process successfully."))?
                        .borrow_mut()
                        .as_inner().id()
                )).output()?;
        }
        Ok(())
    }
}


#[cfg(feature = "gtk")]
impl Ffmpeg {
    // Get file name
    pub fn get_filename(&mut self) -> Result<()> {
        self.saved_filename =
            self.filename
                .0
                .file()
                .ok_or_else(|| anyhow!("Unable to get GFile."))?
                .path()
                .ok_or_else(|| anyhow!("Failed to get path from GFile."))?
                .join(PathBuf::from(format!(
                    "{}.{}",
                    if self.filename.1.text().to_string().trim().eq("") {
                        Utc::now().to_string().replace(" UTC", "").replace(' ', "-")
                    } else {
                        self.filename.1.text().to_string().trim().to_string()
                    },
                    self.filename.2.active_id().ok_or_else(|| anyhow!("Failed to get active_id column."))?
                )))
                .as_path()
                .display()
                .to_string();
        Ok(())
    }

    // Start video recording
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16,  mode: RecordMode) -> Result<()> {
        //if mode == RecordMode::Window && !self.follow_mouse.is_active() {
            // TODO pulse = gstreamer for video  && add to cmd linux + add convert function to gstreamer ouput
        //} else {
        if is_wayland() {
            let filename = self.saved_filename.clone();
            self.output = Path::new(&filename)
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Store the intermediate capture in the system temp dir (/tmp) so it
            // never appears in the user's save folder during recording.
            let video_tempfile = tempfile::Builder::new()
                .prefix(".blue-recorder-video-")
                .suffix(".mkv")
                .tempfile()?
                .keep()?;
            self.temp_video_filename = Path::new(&video_tempfile.1)
                .to_string_lossy()
                .to_string();

            // Start Wayland screen-cast via the XDG portal
            glib::MainContext::default().block_on(self.wayland_recorder.start(
                self.temp_video_filename.clone(),
                match mode {
                    RecordMode::Screen => RecordTypes::Monitor,
                    RecordMode::Window => RecordTypes::Window,
                    _ => RecordTypes::MonitorOrWindow,
                },
                if self.record_mouse.is_active() {
                    CursorModeTypes::Show
                } else {
                    CursorModeTypes::Hidden
                },
                self.record_frames.value() as u16,
            ));

            // If the user closed/cancelled the portal picker, the pipeline was
            // never started — clean up and signal cancellation to the UI.
            if !self.wayland_recorder.is_active() {
                self.temp_video_filename.clear();
                return Err(Error::msg("__cancelled__"));
            }

            // Record audio input to a temp file alongside the Wayland video
            if self.audio_input_switch.is_active() {
                let audio_tempfile = tempfile::Builder::new()
                    .prefix(".blue-recorder-audio-in-")
                    .suffix(".ogg")
                    .tempfile()?
                    .keep()?;
                self.temp_input_audio_filename = Path::new(&audio_tempfile.1)
                    .to_string_lossy()
                    .to_string();
                let mut cmd = FfmpegCommand::new();
                cmd.format("pulse")
                   .input(
                       &self.audio_input_id
                           .active_id()
                           .ok_or_else(|| anyhow!("Failed to get audio input ID."))?,
                   )
                   .format("ogg")
                   .args(["-map_metadata", "-1"])
                   .arg(&self.temp_input_audio_filename)
                   .overwrite();
                if self.audio_record_bitrate.value() as u16 > 0 {
                    cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate.value() as u16)]);
                }
                self.input_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
            }

            // Record audio output to a temp file alongside the Wayland video
            if self.audio_output_switch.is_active() {
                let audio_tempfile = tempfile::Builder::new()
                    .prefix(".blue-recorder-audio-out-")
                    .suffix(".ogg")
                    .tempfile()?
                    .keep()?;
                self.temp_output_audio_filename = Path::new(&audio_tempfile.1)
                    .to_string_lossy()
                    .to_string();
                let mut cmd = FfmpegCommand::new();
                cmd.format("pulse")
                   .input(&self.audio_output_id)
                   .format("ogg")
                   .args(["-map_metadata", "-1"])
                   .arg(&self.temp_output_audio_filename)
                   .overwrite();
                if self.audio_record_bitrate.value() as u16 > 0 {
                    cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate.value() as u16)]);
                }
                self.output_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
            }

            self.width = Some(width);
            self.height = Some(height);
            return Ok(());
        }

        let display = format!("{}+{},{}",
                              std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                              .as_str(),
                              x,
                              y
        );
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "x11grab";
        self.width = Some(width);
        self.height = Some(height);
        let filename = self.saved_filename.clone();
        self.output = Path::new(&filename).extension()
                                          .ok_or_else(|| anyhow!("Failed to get file extension."))?
                                          .to_string_lossy().to_string();

        // Record video to tmp if audio record enabled
        if self.output == "gif" {
            let suffix = ".mp4";
            let video_tempfile = tempfile::Builder::new().prefix(".ffmpeg-video-")
                                                         .suffix(suffix)
                                                         .tempfile()?
                                                         .keep()?;
            self.temp_video_filename = Path::new(&video_tempfile.1).to_string_lossy()
                                                                   .to_string();
        }

        // Record video with specified width and hight
        if self.follow_mouse.is_active() {
            match mode {
                RecordMode::Screen => {
                    let width = width as f32 * 0.95;
                    let height = height as f32 * 0.95;
                    ffmpeg_command.size(width as u32, height as u32);
                },
                _=> {
                    ffmpeg_command.size(width as u32, height as u32);
                }
            }
        } else {
            ffmpeg_command.size(width as u32, height as u32);
        }

        // Show grabbed area
        if self.show_area.is_active() {
            ffmpeg_command.args(["-show_region", "1"]);
        }

        // If show mouse switch is enabled, draw the mouse to video
        if self.record_mouse.is_active() {
            ffmpeg_command.args(["-draw_mouse", "1"]);
        } else {
            ffmpeg_command.args(["-draw_mouse", "0"]);
        };

        // Follow the mouse
        if self.follow_mouse.is_active() {
            ffmpeg_command.args(["-follow_mouse", "centered"]);
        }

        // Disable frame rate if value is zero
        if self.record_frames.value() as u16 > 0 {
            ffmpeg_command.args(["-framerate", &self.record_frames.value().to_string()]);
        }

        // Video format && input
        ffmpeg_command.format(format)
                      .input(display);

        // Record audio input
        if self.audio_input_switch.is_active() {
            ffmpeg_command.format("pulse")
                          .input(&self.audio_input_id.active_id()
                                 .ok_or_else(|| anyhow!("Failed to get audio input ID."))?
                          );
        }

        // Record audio output
        if self.audio_output_switch.is_active() {
            ffmpeg_command.format("pulse")
                          .input(&self.audio_output_id);
        }

        // Disable video bitrate if value is zero
        if self.video_record_bitrate.value() as u16 > 0 {
            ffmpeg_command.args([
                "-b:v",
                &format!("{}K", self.video_record_bitrate.value() as u16),
            ]);
        }

        // Disable audio bitrate if value is zero
        if self.audio_input_switch.is_active() || self.audio_output_switch.is_active() {
            if self.audio_record_bitrate.value() as u16 > 0 {
                ffmpeg_command.args([
                    "-b:a",
                    &format!("{}K", self.audio_record_bitrate.value() as u16),
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
        sleep(Duration::from_secs(self.record_delay.value() as u64));

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
        } else if self.video_switch.is_active() && is_wayland() {
            // Signal audio processes to stop (non-blocking).
            if let Some(proc) = self.input_audio_process.clone() {
                let _ = proc.borrow_mut().quit();
            }
            if let Some(proc) = self.output_audio_process.clone() {
                let _ = proc.borrow_mut().quit();
            }
            // Stop GStreamer pipeline (runs the GLib main loop while waiting).
            glib::MainContext::default().block_on(self.wayland_recorder.stop());

            // Wait for audio processes to fully exit before merging.
            // We sent SIGTERM above; GStreamer stop takes 1-2 s which is usually
            // enough, but on slower systems ffmpeg may still be flushing the OGG
            // container. poll with try_wait() (non-blocking, no pipe-deadlock)
            // for up to 5 s to ensure the audio files are completely written.
            let audio_deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let in_done = self.input_audio_process.as_ref()
                    .map(|p| p.borrow_mut().as_inner_mut().try_wait()
                         .ok().flatten().is_some())
                    .unwrap_or(true);
                let out_done = self.output_audio_process.as_ref()
                    .map(|p| p.borrow_mut().as_inner_mut().try_wait()
                         .ok().flatten().is_some())
                    .unwrap_or(true);
                if (in_done && out_done) || std::time::Instant::now() >= audio_deadline {
                    break;
                }
                sleep(Duration::from_millis(50));
            }
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
        Ok(())
    }

    // Start audio input recording
    pub fn start_input_audio(&mut self) -> Result<()> {
        let mut ffmpeg_command = FfmpegCommand::new();
        ffmpeg_command.format("pulse")
                      .input(&self.audio_input_id.active_id()
                             .ok_or_else(|| anyhow!("Failed to get audio input ID."))?
                      );
        ffmpeg_command.format("ogg");
        if self.audio_output_switch.is_active() {
            ffmpeg_command.format("pulse")
                          .input(&self.audio_output_id);
            ffmpeg_command.format("ogg");
        }

        // Disable bitrate if value is zero
        if self.audio_record_bitrate.value() as u16 > 0 {
            ffmpeg_command.args([
                "-b:a",
                &format!("{}K", self.audio_record_bitrate.value() as u16),
            ]);
        }

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        ffmpeg_command.arg(&self.saved_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !self.video_switch.is_active() {
            sleep(Duration::from_secs(self.record_delay.value() as u64));
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
        ffmpeg_command.format("pulse")
                      .input(&self.audio_output_id)
                      .format("ogg");
        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        ffmpeg_command.arg(&self.saved_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !self.video_switch.is_active() && !self.audio_input_switch.is_active() {
            sleep(Duration::from_secs(self.record_delay.value() as u64));
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
        if !is_video_record(&self.temp_video_filename) {
            return Ok(());
        }

        // Do NOT call validate_video_file here — it runs a nested GLib main loop
        // while we're inside a borrow_mut(), which risks a double-borrow panic.
        // Let ffmpeg handle invalid input gracefully instead.

        if self.output == "gif" {
            let filter = format!(
                "fps={fps},scale={h}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                fps = self.record_frames.value() as u16,
                h = self.height.ok_or_else(|| anyhow!("Unable to get height value"))?,
            );
            let cmd = format!(
                "ffmpeg -i file:{src} -filter_complex '{filter}' -loop 0 {dst} -y",
                src = self.temp_video_filename,
                dst = self.saved_filename,
            );
            std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .map_err(|e| Error::msg(format!("{}", e)))?;
            return Ok(());
        }

        // Non-GIF: transcode/copy temp file → final container, optionally with audio.

        // Abort early with a clear message if GStreamer produced nothing.
        let source_bytes = std::fs::metadata(&self.temp_video_filename)
            .map(|m| m.len())
            .unwrap_or(0);
        if source_bytes == 0 {
            return Err(Error::msg(
                "The captured video file is empty — the GStreamer recording pipeline \
                 did not produce any data. Check that PipeWire and the screen-cast \
                 portal are working correctly."
            ));
        }

        let has_input_audio = !self.temp_input_audio_filename.is_empty()
            && Path::new(&self.temp_input_audio_filename).exists();
        let has_output_audio = !self.temp_output_audio_filename.is_empty()
            && Path::new(&self.temp_output_audio_filename).exists();

        let audio_args: Vec<String> = if has_input_audio || has_output_audio {
            let br = self.audio_record_bitrate.value() as u16;
            if br > 0 {
                vec!["-c:a".into(), "aac".into(), "-b:a".into(), format!("{}K", br)]
            } else {
                vec!["-c:a".into(), "aac".into()]
            }
        } else {
            vec![]
        };

        // For webm/mkv the captured codec (VP9 or H.264) can be copied as-is.
        // For every other container (mp4, avi, wmv, nut …) we must transcode
        // to a codec the container actually expects — never copy, because that
        // would keep VP9 inside an mp4, which players report as "WebM video".
        let video_codecs: Vec<&str> = match self.output.as_str() {
            "webm" | "mkv" => vec!["copy"],
            _ => vec!["libx264", "libx265", "mpeg4"],
        };

        // Use std::process::Command (not FfmpegCommand) so that .output() reads
        // stdout+stderr to completion — FfmpegChild::wait() can deadlock when
        // ffmpeg writes error messages faster than we drain the pipe.
        let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();

        for codec in &video_codecs {
            let mut args: Vec<String> = vec![
                "-i".into(), self.temp_video_filename.clone(),
            ];
            if has_input_audio  { args.extend(["-i".into(), self.temp_input_audio_filename.clone()]); }
            if has_output_audio { args.extend(["-i".into(), self.temp_output_audio_filename.clone()]); }

            args.extend(["-c:v".into(), (*codec).into()]);
            match *codec {
                "libx264" | "libx265" => args.extend(["-preset".into(), "fast".into(), "-crf".into(), "23".into()]),
                "mpeg4"               => args.extend(["-qscale:v".into(), "3".into()]),
                _                     => {}
            }
            args.extend(audio_args.clone());
            args.extend(["-map_metadata".into(), "-1".into()]);
            args.push(self.saved_filename.clone());
            args.push("-y".into());

            let _ = std::fs::remove_file(&self.saved_filename);
            let _ = std::process::Command::new(&ffmpeg_bin).args(&args).output();

            if Path::new(&self.saved_filename).exists() {
                return Ok(());
            }
        }

        // Absolute last resort: copy the raw capture so the recording is never lost.
        // Prefix with '.' to keep it hidden until the user explicitly opens it.
        let stem = Path::new(&self.saved_filename)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let webm_name = format!(".{}.webm", stem);
        let webm_path = Path::new(&self.saved_filename)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&webm_name)
            .to_string_lossy()
            .to_string();
        if std::fs::copy(&self.temp_video_filename, &webm_path).is_ok() {
            self.saved_filename = webm_path;
            return Ok(());
        }

        Err(Error::msg(
            "Failed to encode the recording. Please install ffmpeg with libx264 \
             or mpeg4 support (any standard ffmpeg package includes mpeg4)."
        ))
    }

    // Clean tmp files
    pub fn clean(&mut self) -> Result<()> {
        for tmp in [
            &self.temp_video_filename,
            &self.temp_input_audio_filename,
            &self.temp_output_audio_filename,
        ] {
            if !tmp.is_empty() && Path::new(tmp).try_exists()? {
                std::fs::remove_file(tmp)?;
            }
        }
        Ok(())
    }

    // Kill process
    pub fn kill(&mut self) -> Result<()> {
        if self.video_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.video_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the video recording process successfully."))?
                        .borrow_mut().as_inner().id()
                )).output()?;
        }
        if self.input_audio_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.input_audio_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the intput audio recording process successfully."))?
                        .borrow_mut().as_inner().id()
                )).output()?;
        }
        if self.output_audio_process.is_some() {
            std::process::Command::new("kill")
                .arg(format!(
                    "{}",
                    self.output_audio_process
                        .clone()
                        .ok_or_else(|| anyhow!("Unable to kill the output audio recording process successfully."))?
                        .borrow_mut().as_inner().id()
                )).output()?;
        }
        Ok(())
    }
}
