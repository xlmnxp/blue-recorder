use anyhow::Result;
use std::process::Command;

// Select recording mode
#[derive(Clone, Copy)]
pub enum RecordMode {
    Area,
    Screen,
    Window,
}

#[cfg(feature = "gtk")]
// Disable GtkWidget
pub fn disable_input_widgets(input_widgets: Vec<adw::gtk::Widget>) {
    use adw::gtk::prelude::WidgetExt;
    for widget in input_widgets {
        widget.set_sensitive(false);
    }
}

#[cfg(feature = "gtk")]
// Enable GtkWidget
pub fn enable_input_widgets(input_widgets: Vec<adw::gtk::Widget>) {
    use adw::gtk::prelude::WidgetExt;
    for widget in input_widgets {
        widget.set_sensitive(true);
    }
}

#[cfg(feature = "gtk")]
// Execute command after finish recording
pub fn exec(command: &str) -> Result<()> {
    if !command.trim().is_empty() {
        subprocess::Exec::shell(command.trim()).popen()?;
    }
    Ok(())
}

// Check if tmp input video file exist
pub fn is_input_audio_record(audio_filename: &str) -> bool {
    std::path::Path::new(audio_filename).exists()
}

// Check if tmp output video file exist
pub fn is_output_audio_record(audio_filename: &str) -> bool {
    std::path::Path::new(audio_filename).exists()
}

#[cfg(feature = "gtk")]
// Overwrite file if exists or not
pub fn is_overwrite(msg_bundle: &str, filename: &str, window: adw::Window) -> bool {
    let is_file_already_exists = std::path::Path::new(filename).exists();
    if is_file_already_exists {
        let message_dialog = adw::gtk::MessageDialog::new(
                Some(&window),
                adw::gtk::DialogFlags::all(),
                adw::gtk::MessageType::Warning,
                adw::gtk::ButtonsType::YesNo,
                msg_bundle,
        );

        let main_context = glib::MainContext::default();
        use adw::prelude::*;
        let answer = main_context.block_on(message_dialog.run_future());
        message_dialog.close();

        if answer != adw::gtk::ResponseType::Yes {
            return false;
        } else {
            return true;
        }
    } else {
        return true;
    }
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

// Get audio output source
#[cfg(feature = "gtk")]
pub fn audio_output_source() -> Result<String> {
    // Get the default sink
    let default_sink_output = Command::new("pactl")
        .arg("get-default-sink")
        .output()?;

    let default_sink = String::from_utf8_lossy(&default_sink_output.stdout)
        .trim()
        .to_string();

    // List sinks and filter for the monitor of the default sink
    let sinks_output = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .output()?;

    let sinks = String::from_utf8_lossy(&sinks_output.stdout);
    let monitor_line = sinks
        .lines()
        .find(|line| line.contains(&format!("{}.monitor", default_sink)))
        .unwrap_or("");

    // Extract the part after the colon
    let output_source = monitor_line.split(':')
        .nth(1)
        .unwrap_or("")
        .trim().to_string();
    Ok(output_source)
}

#[cfg(feature = "gtk")]
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

#[cfg(feature = "gtk")]
// Get audio input source list
pub fn sources_descriptions_list() -> Result<Vec<String>> {
    let sources_descriptions: Vec<String> = {
        let list_sources_child = std::process::Command::new("pactl")
            .args(["list", "sources"])
            .stdout(std::process::Stdio::piped())
            .spawn();
        let sources_descriptions = String::from_utf8(if let Ok(..) = list_sources_child {
            std::process::Command::new("grep")
                .args(["-e", "device.description"])
                .stdin(list_sources_child?.stdout.take()
                       .ok_or_else(|| anyhow::anyhow!("Failed to get audio input source descriptions."))?)
                .output()?
                .stdout
        } else {
            Vec::new()
        })?;
        sources_descriptions
            .split('\n')
            .map(|s| {
                s.trim()
                 .replace("device.description = ", "")
                 .replace('\"', "")
            })
            .filter(|s| !s.is_empty())
            .collect()
    };
    Ok(sources_descriptions)
}
