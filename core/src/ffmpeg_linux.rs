use anyhow::{anyhow, Error, Result};
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::utils::{is_video_record, is_wayland, RecordMode};

#[cfg(any(target_os = "freebsd", target_os = "linux"))]
use crate::wayland_linux::{CursorModeTypes, RecordTypes, WaylandRecorder};

/// All recording configuration and runtime state in one plain-data struct.
/// GUI layers populate the configuration fields before calling start_* methods;
/// core only reads plain Rust types and never touches GTK widgets.
#[derive(Clone)]
pub struct Ffmpeg {
    // ── Configuration — set by the caller before each recording ──────────
    pub audio_input_id: String,
    pub audio_output_id: String,
    /// Final output path.  GUI computes this from its folder/filename/format
    /// widgets and writes it here before calling start_video / start_*_audio.
    pub filename: String,
    /// File extension derived from `filename` (e.g. "mp4").
    pub output: String,
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

    // ── Runtime state — managed internally ───────────────────────────────
    /// Actual path written; may differ from `filename` after a fallback.
    pub saved_filename: String,
    pub temp_video_filename: String,
    pub temp_input_audio_filename: String,
    pub temp_output_audio_filename: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub input_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub output_audio_process: Option<Rc<RefCell<FfmpegChild>>>,
    pub video_process: Option<Rc<RefCell<FfmpegChild>>>,

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    pub wayland_recorder: WaylandRecorder,
}

impl Ffmpeg {
    pub fn start_video(&mut self, x: u16, y: u16, width: u16, height: u16, mode: RecordMode) -> Result<()> {
        self.saved_filename = self.filename.clone();
        self.output = Path::new(&self.filename)
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        if is_wayland() {
            let video_tempfile = tempfile::Builder::new()
                .prefix(".blue-recorder-video-")
                .suffix(".mkv")
                .tempfile()?
                .keep()?;
            self.temp_video_filename = Path::new(&video_tempfile.1).to_string_lossy().to_string();

            async_std::task::block_on(self.wayland_recorder.start(
                self.temp_video_filename.clone(),
                match mode {
                    RecordMode::Screen => RecordTypes::Monitor,
                    RecordMode::Window => RecordTypes::Window,
                    _ => RecordTypes::MonitorOrWindow,
                },
                if self.record_mouse { CursorModeTypes::Show } else { CursorModeTypes::Hidden },
                self.record_frames,
            ));

            if !self.wayland_recorder.is_active() {
                self.temp_video_filename.clear();
                return Err(Error::msg("__cancelled__"));
            }

            if self.audio_input_enabled {
                let tf = tempfile::Builder::new()
                    .prefix(".blue-recorder-audio-in-").suffix(".ogg")
                    .tempfile()?.keep()?;
                self.temp_input_audio_filename = Path::new(&tf.1).to_string_lossy().to_string();
                let mut cmd = FfmpegCommand::new();
                cmd.format("pulse").input(&self.audio_input_id)
                   .format("ogg").args(["-map_metadata", "-1"])
                   .arg(&self.temp_input_audio_filename).overwrite();
                if self.audio_record_bitrate > 0 {
                    cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate)]);
                }
                self.input_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
            }

            if self.audio_output_enabled {
                let tf = tempfile::Builder::new()
                    .prefix(".blue-recorder-audio-out-").suffix(".ogg")
                    .tempfile()?.keep()?;
                self.temp_output_audio_filename = Path::new(&tf.1).to_string_lossy().to_string();
                let mut cmd = FfmpegCommand::new();
                cmd.format("pulse").input(&self.audio_output_id)
                   .format("ogg").args(["-map_metadata", "-1"])
                   .arg(&self.temp_output_audio_filename).overwrite();
                if self.audio_record_bitrate > 0 {
                    cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate)]);
                }
                self.output_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
            }

            self.width  = Some(width);
            self.height = Some(height);
            return Ok(());
        }

        let display = format!(
            "{}+{},{}",
            std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string()),
            x, y,
        );
        let mut cmd = FfmpegCommand::new();
        self.width  = Some(width);
        self.height = Some(height);

        if self.output == "gif" {
            let tf = tempfile::Builder::new()
                .prefix(".ffmpeg-video-").suffix(".mp4")
                .tempfile()?.keep()?;
            self.temp_video_filename = Path::new(&tf.1).to_string_lossy().to_string();
        }

        if self.follow_mouse {
            match mode {
                RecordMode::Screen => {
                    cmd.size((width as f32 * 0.95) as u32, (height as f32 * 0.95) as u32);
                }
                _ => { cmd.size(width as u32, height as u32); }
            }
        } else {
            cmd.size(width as u32, height as u32);
        }

        if self.show_area   { cmd.args(["-show_region", "1"]); }
        if self.record_mouse { cmd.args(["-draw_mouse", "1"]); } else { cmd.args(["-draw_mouse", "0"]); }
        if self.follow_mouse { cmd.args(["-follow_mouse", "centered"]); }
        if self.record_frames > 0 {
            cmd.args(["-framerate", &self.record_frames.to_string()]);
        }

        cmd.format("x11grab").input(display);

        if self.audio_input_enabled  { cmd.format("pulse").input(&self.audio_input_id); }
        if self.audio_output_enabled { cmd.format("pulse").input(&self.audio_output_id); }

        if self.video_record_bitrate > 0 {
            cmd.args(["-b:v", &format!("{}K", self.video_record_bitrate)]);
        }
        if (self.audio_input_enabled || self.audio_output_enabled) && self.audio_record_bitrate > 0 {
            cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate)]);
        }

        cmd.args(["-map_metadata", "-1"]);
        cmd.args([if self.output == "gif" { &self.temp_video_filename } else { &self.saved_filename }]);
        cmd.overwrite();

        sleep(Duration::from_secs(self.record_delay as u64));
        self.video_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
        Ok(())
    }

    pub fn stop_video(&mut self) -> Result<()> {
        if self.video_process.is_some() {
            match self.video_process.clone()
                .ok_or_else(|| anyhow!("Not exiting the video recording process successfully."))?
                .borrow_mut().quit()
            {
                Ok(_) => {
                    if self.output == "gif" {
                        match self.merge() {
                            Ok(_)  => self.clean()?,
                            Err(e) => { self.clean()?; return Err(Error::msg(format!("{}", e))); }
                        }
                    }
                }
                Err(e) => {
                    if self.output == "gif" { self.clean()?; }
                    else { self.temp_video_filename = self.saved_filename.clone(); self.clean()?; }
                    return Err(Error::msg(format!("{}", e)));
                }
            }
        }

        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        if self.video_enabled && is_wayland() {
            if let Some(p) = self.input_audio_process.clone()  { let _ = p.borrow_mut().quit(); }
            if let Some(p) = self.output_audio_process.clone() { let _ = p.borrow_mut().quit(); }

            async_std::task::block_on(self.wayland_recorder.stop());

            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let in_done  = self.input_audio_process.as_ref()
                    .map(|p| p.borrow_mut().as_inner_mut().try_wait().ok().flatten().is_some())
                    .unwrap_or(true);
                let out_done = self.output_audio_process.as_ref()
                    .map(|p| p.borrow_mut().as_inner_mut().try_wait().ok().flatten().is_some())
                    .unwrap_or(true);
                if (in_done && out_done) || std::time::Instant::now() >= deadline { break; }
                sleep(Duration::from_millis(50));
            }

            match self.merge() {
                Ok(_)  => self.clean()?,
                Err(e) => { self.clean()?; return Err(Error::msg(format!("{}", e))); }
            }
        }

        Ok(())
    }

    pub fn start_input_audio(&mut self) -> Result<()> {
        let mut cmd = FfmpegCommand::new();
        cmd.format("pulse").input(&self.audio_input_id).format("ogg");
        if self.audio_output_enabled {
            cmd.format("pulse").input(&self.audio_output_id).format("ogg");
        }
        if self.audio_record_bitrate > 0 {
            cmd.args(["-b:a", &format!("{}K", self.audio_record_bitrate)]);
        }
        cmd.args(["-map_metadata", "-1"]).arg(&self.saved_filename).overwrite();
        if !self.video_enabled {
            sleep(Duration::from_secs(self.record_delay as u64));
        }
        self.input_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
        Ok(())
    }

    pub fn stop_input_audio(&mut self) -> Result<()> {
        if self.input_audio_process.is_some() {
            match self.input_audio_process.clone()
                .ok_or_else(|| anyhow!("Not exiting the input audio recording process successfully."))?
                .borrow_mut().quit()
            {
                Ok(_) => {}
                Err(e) => {
                    self.temp_video_filename = self.saved_filename.clone();
                    self.clean()?;
                    return Err(Error::msg(format!("{}", e)));
                }
            }
        }
        Ok(())
    }

    pub fn start_output_audio(&mut self) -> Result<()> {
        let mut cmd = FfmpegCommand::new();
        cmd.format("pulse").input(&self.audio_output_id).format("ogg");
        cmd.args(["-map_metadata", "-1"]).arg(&self.saved_filename).overwrite();
        if !self.video_enabled && !self.audio_input_enabled {
            sleep(Duration::from_secs(self.record_delay as u64));
        }
        self.output_audio_process = Some(Rc::new(RefCell::new(cmd.spawn()?)));
        Ok(())
    }

    pub fn stop_output_audio(&mut self) -> Result<()> {
        if self.output_audio_process.is_some() {
            match self.output_audio_process.clone()
                .ok_or_else(|| anyhow!("Not exiting the output audio recording process successfully."))?
                .borrow_mut().quit()
            {
                Ok(_) => {}
                Err(e) => {
                    self.temp_video_filename = self.saved_filename.clone();
                    self.clean()?;
                    return Err(Error::msg(format!("{}", e)));
                }
            }
        }
        Ok(())
    }

    pub fn merge(&mut self) -> Result<()> {
        if !is_video_record(&self.temp_video_filename) {
            return Ok(());
        }

        if self.output == "gif" {
            let filter = format!(
                "fps={fps},scale={h}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                fps = self.record_frames,
                h   = self.height.ok_or_else(|| anyhow!("Unable to get height value"))?,
            );
            let cmd = format!(
                "ffmpeg -i file:{src} -filter_complex '{filter}' -loop 0 {dst} -y",
                src = self.temp_video_filename,
                dst = self.saved_filename,
            );
            std::process::Command::new("sh").arg("-c").arg(&cmd).output()
                .map_err(|e| Error::msg(format!("{}", e)))?;
            return Ok(());
        }

        let source_bytes = std::fs::metadata(&self.temp_video_filename).map(|m| m.len()).unwrap_or(0);
        if source_bytes == 0 {
            return Err(Error::msg(
                "The captured video file is empty — the GStreamer recording pipeline \
                 did not produce any data. Check that PipeWire and the screen-cast \
                 portal are working correctly."
            ));
        }

        let has_input_audio  = !self.temp_input_audio_filename.is_empty()
            && Path::new(&self.temp_input_audio_filename).exists();
        let has_output_audio = !self.temp_output_audio_filename.is_empty()
            && Path::new(&self.temp_output_audio_filename).exists();

        let audio_codec = if self.output == "webm" { "libopus" } else { "aac" };
        let audio_args: Vec<String> = if has_input_audio || has_output_audio {
            if self.audio_record_bitrate > 0 {
                vec!["-c:a".into(), audio_codec.into(), "-b:a".into(), format!("{}K", self.audio_record_bitrate)]
            } else {
                vec!["-c:a".into(), audio_codec.into()]
            }
        } else { vec![] };

        let video_codecs: Vec<&str> = match self.output.as_str() {
            "webm" | "mkv" => vec!["copy"],
            _              => vec!["libx264", "libx265", "mpeg4"],
        };

        let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();
        for codec in &video_codecs {
            let mut args: Vec<String> = vec!["-i".into(), self.temp_video_filename.clone()];
            if has_input_audio  { args.extend(["-i".into(), self.temp_input_audio_filename.clone()]); }
            if has_output_audio { args.extend(["-i".into(), self.temp_output_audio_filename.clone()]); }
            args.extend(["-c:v".into(), (*codec).into()]);
            match *codec {
                "libx264" | "libx265" => args.extend(["-preset".into(), "fast".into(), "-crf".into(), "23".into()]),
                "mpeg4"               => args.extend(["-qscale:v".into(), "3".into()]),
                _ => {}
            }
            args.extend(audio_args.clone());
            args.extend(["-map_metadata".into(), "-1".into()]);
            args.push(self.saved_filename.clone());
            args.push("-y".into());

            let _ = std::fs::remove_file(&self.saved_filename);
            let _ = std::process::Command::new(&ffmpeg_bin).args(&args).output();
            if Path::new(&self.saved_filename).exists() { return Ok(()); }
        }

        // Last resort: preserve raw capture as hidden .webm
        let stem = Path::new(&self.saved_filename).file_stem().unwrap_or_default().to_string_lossy();
        let webm_path = Path::new(&self.saved_filename)
            .parent().unwrap_or_else(|| Path::new("."))
            .join(format!(".{}.webm", stem))
            .to_string_lossy().to_string();
        if std::fs::copy(&self.temp_video_filename, &webm_path).is_ok() {
            self.saved_filename = webm_path;
            return Ok(());
        }

        Err(Error::msg(
            "Failed to encode the recording. Please install ffmpeg with libx264 \
             or mpeg4 support (any standard ffmpeg package includes mpeg4)."
        ))
    }

    pub fn clean(&mut self) -> Result<()> {
        for tmp in [&self.temp_video_filename, &self.temp_input_audio_filename, &self.temp_output_audio_filename] {
            if !tmp.is_empty() && Path::new(tmp).try_exists()? {
                std::fs::remove_file(tmp)?;
            }
        }
        Ok(())
    }

    pub fn kill(&mut self) -> Result<()> {
        for proc in [&self.video_process, &self.input_audio_process, &self.output_audio_process] {
            if let Some(p) = proc {
                let _ = std::process::Command::new("kill")
                    .arg(p.borrow_mut().as_inner().id().to_string())
                    .output();
            }
        }
        Ok(())
    }
}
