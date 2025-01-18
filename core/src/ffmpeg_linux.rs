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
use std::{cell::RefCell, time::Instant};
use std::path::Path;
#[cfg(feature = "gtk")]
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::utils::{is_input_audio_record, is_output_audio_record, is_valide, is_video_record, RecordMode};

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
    pub temp_input_audio_filename: String,
    pub temp_output_audio_filename: String,
    pub temp_video_filename: String,
    pub saved_filename: String,
    pub height: Option<u16>,
    pub input_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub output_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub audio_record_bitrate: SpinButton,
    pub record_delay: SpinButton,
    pub record_frames: SpinButton,
    pub video_record_bitrate: SpinButton,
    pub follow_mouse: CheckButton,
    pub record_mouse: CheckButton,
    pub show_area: CheckButton,
}

#[cfg(feature = "cmd")]
impl Ffmpeg {
    // Start video recording
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16,  mode: RecordMode) -> Result<()> {
        let display = format!("{}+{},{}",
                std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                .as_str(),
                x,
                y
        );
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "x11grab";
        self.height = Some(height);

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

        // Record video with specified width and hight
        if self.follow_mouse {
            match mode {
                RecordMode::Screen => {
                    let width = width as f32 * 0.95;
                    let height = height as f32 * 0.95;
                    ffmpeg_command.size(width as u32, height as u32);
                },
                _=> {
                    ffmpeg_command.size(width.into(), height.into());
                }
            }
        } else {
            ffmpeg_command.size(width.into(), height.into());
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
        let input_audio_tempfile = tempfile::Builder::new().prefix("ffmpeg-audio-")
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
        let output_audio_tempfile = tempfile::Builder::new().prefix("ffmpeg-audio-")
                                                            .suffix(".ogg")
                                                            .tempfile()?
                                                            .keep()?;
        self.temp_output_audio_filename = Path::new(&output_audio_tempfile.1).to_string_lossy()
                                                                             .to_string();
        let mut ffmpeg_command = FfmpegCommand::new();
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
        //if mode == RecordMode::Window && !self.follow_mouse.is_active() { //REVIEW //TODO
            // pulse = gstreamer for video  && add to cmd linux + add convert function to gstreamer ouput
        //} else {}
        let display = format!("{}+{},{}",
                std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string())
                .as_str(),
                x,
                y
        );
        let mut ffmpeg_command = FfmpegCommand::new();
        let format = "x11grab";
        self.height = Some(height);
        let filename = self.saved_filename.clone();
        self.output = Path::new(&filename).extension()
                                          .ok_or_else(|| anyhow!("Failed to get file extension."))?
                                          .to_string_lossy().to_string();

        // Record video to tmp if audio record enabled
        if !self.audio_input_id.active_id().ok_or_else(|| anyhow!("Failed to get audio input device ID."))?
                                           .to_string().is_empty()
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

        // Record video with specified width and hight
        if self.follow_mouse.is_active() {
            match mode {
                RecordMode::Screen => {
                    let width = width as f32 * 0.95;
                    let height = height as f32 * 0.95;
                    ffmpeg_command.size(width as u32, height as u32);
                },
                _=> {
                    ffmpeg_command.size(width.into(), height.into());
                }
            }
        } else {
            ffmpeg_command.size(width.into(), height.into());
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

        // Disable bitrate if value is zero
        if self.video_record_bitrate.value() as u16 > 0 {
            ffmpeg_command.args([
                "-b:v",
                &format!("{}K", self.video_record_bitrate.value() as u16),
            ]);
        }

        // tmp file
        if self.audio_input_id.active_id().ok_or_else(|| anyhow!("Failed to get audio input device ID."))?
                                          .to_string().is_empty() &&
            self.audio_output_id.is_empty() &&
            self.output != "gif"
        {
            ffmpeg_command.args(["-hls_flags", "temp_file"]);
        }

        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);

        // Output
        let saved_filename = self.saved_filename.clone();
        ffmpeg_command.args([
            {
                if !self.audio_input_id.active_id()
                                       .ok_or_else(|| anyhow!("Failed to get active audio input device ID."))?
                                       .to_string().is_empty()
                    || !self.audio_output_id.is_empty()
                    || self.output == "gif"
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
        // Kill the process to stop recording
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
        ffmpeg_command.format("pulse")
                      .input(&self.audio_input_id.active_id()
                             .ok_or_else(|| anyhow!("Failed to get audio input ID."))?
                      )
                      .format("ogg");
        // Disable bitrate if value is zero
        if self.audio_record_bitrate.value() as u16 > 0 {
            ffmpeg_command.args([
                "-b:a",
                &format!("{}K", self.audio_record_bitrate.value() as u16),
            ]);
        }
        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);
        ffmpeg_command.arg(&self.temp_input_audio_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !is_video_record(&self.temp_video_filename) {
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
        ffmpeg_command.format("pulse")
                      .input(&self.audio_output_id)
                      .format("ogg");
        // Remove metadate
        ffmpeg_command.args(["-map_metadata", "-1"]);
        ffmpeg_command.arg(&self.temp_output_audio_filename);
        ffmpeg_command.overwrite();

        // Sleep for delay
        if !is_video_record(&self.temp_video_filename) && !is_input_audio_record(&self.temp_input_audio_filename) {
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
                                     self.record_frames.value() as u16,
                                     self.height.ok_or_else
                                     (|| anyhow!("Unable to get height value"))?);
                let ffmpeg_convert = format!("ffmpeg -i file:{} -filter_complex '{}' \
                                              -loop 0 {} -y", &self.temp_video_filename,filter,
                                             &self.saved_filename
                                             .clone());
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
                &self.saved_filename
                     .clone()
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
            ffmpeg_command.arg(&self.saved_filename
                               .clone())
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
