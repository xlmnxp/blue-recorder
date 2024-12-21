use anyhow::Result;
use std::process::Command;

// Select recording mode
#[derive(Clone, Copy)]
pub enum RecordMode {
    Area,
    Screen,
    Window,
}

// Check if tmp input video file exist
pub fn is_input_audio_record(audio_filename: &str) -> bool {
    std::path::Path::new(audio_filename).exists()
}

// Check if tmp output video file exist
pub fn is_output_audio_record(audio_filename: &str) -> bool {
    std::path::Path::new(audio_filename).exists()
}

// Detect if snap package is used
pub fn is_snap() -> bool {
    !std::env::var("SNAP").unwrap_or_default().is_empty()
}

// Validate audio/video file integrity
pub fn is_valide(filename: &str) -> Result<bool> {
    let validate = Command::new("ffmpeg")
        .args(["-v", "error",
               "-i", filename,
               "-f", "null", "-"
        ]).output()?;
    if validate.status.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Check if tmp video file exist
pub fn is_video_record(video_filename: &str) -> bool {
    std::path::Path::new(video_filename).exists()
}

// Detect wayland session
pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .eq_ignore_ascii_case("wayland")
}

// Play recorded file
pub fn play_record(file_name: &str) -> Result<()> {
    if is_snap() {
        // open the video using snapctrl for snap package
        Command::new("snapctl").arg("user-open")
                               .arg(file_name)
                               .spawn()?;
    } else {
        open::that(file_name)?;
    }
    Ok(())
}
