#[cfg(any(target_os = "freebsd", target_os = "linux"))]
pub mod ffmpeg_linux;

#[cfg(target_os = "windows")]
pub mod ffmpeg_windows;

pub mod utils;

#[cfg(any(target_os = "freebsd", target_os = "linux"))]
pub mod wayland_linux;