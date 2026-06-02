use anyhow::Result;
use std::process::Command;

#[derive(Clone, Copy)]
pub enum RecordMode {
    Area,
    Screen,
    Window,
}

pub fn is_input_audio_record(filename: &str) -> bool {
    std::path::Path::new(filename).exists()
}

pub fn is_output_audio_record(filename: &str) -> bool {
    std::path::Path::new(filename).exists()
}

pub fn is_snap() -> bool {
    !std::env::var("SNAP").unwrap_or_default().is_empty()
}

pub fn is_valid(filename: &str) -> Result<bool> {
    let out = Command::new("ffmpeg")
        .args(["-v", "error", "-i", filename, "-c", "copy", "-f", "null", "-"])
        .output()?;
    Ok(out.status.success())
}

pub fn is_video_record(filename: &str) -> bool {
    std::path::Path::new(filename).exists()
}

pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .eq_ignore_ascii_case("wayland")
}

pub fn play_record(file_name: &str) -> Result<()> {
    if is_snap() {
        Command::new("snapctl").arg("user-open").arg(file_name).spawn()?;
    } else {
        open::that(file_name)?;
    }
    Ok(())
}
