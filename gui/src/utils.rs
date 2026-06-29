use adw::gtk::{Widget, MessageDialog, DialogFlags, MessageType, ButtonsType};
use adw::gtk::prelude::*;
use adw::Window;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub fn audio_output_source() -> Result<String> {
    let sink = Command::new("pactl").arg("get-default-sink").output()?;
    let default_sink = String::from_utf8_lossy(&sink.stdout).trim().to_string();

    let sinks = Command::new("pactl").arg("list").arg("sinks").output()?;
    let sinks_str = String::from_utf8_lossy(&sinks.stdout);
    let monitor_line = sinks_str
        .lines()
        .find(|l| l.contains(&format!("{}.monitor", default_sink)))
        .unwrap_or("");
    let source = monitor_line.split(':').nth(1).unwrap_or("").trim().to_string();
    Ok(source)
}

pub fn disable_input_widgets(widgets: Vec<Widget>) {
    for w in widgets { w.set_sensitive(false); }
}

pub fn enable_input_widgets(widgets: Vec<Widget>) {
    for w in widgets { w.set_sensitive(true); }
}


pub fn is_overwrite(msg: &str, filename: &str, window: Window) -> bool {
    if !std::path::Path::new(filename).exists() {
        return true;
    }
    let dialog = MessageDialog::new(
        Some(&window),
        DialogFlags::all(),
        MessageType::Warning,
        ButtonsType::YesNo,
        msg,
    );
    let answer = glib::MainContext::default().block_on(dialog.run_future());
    dialog.close();
    answer == adw::gtk::ResponseType::Yes
}

pub fn sources_descriptions_list() -> Result<Vec<String>> {
    let child = Command::new("pactl")
        .args(["list", "sources"])
        .stdout(std::process::Stdio::piped())
        .spawn();
    let raw = if let Ok(mut c) = child {
        Command::new("grep")
            .args(["-e", "device.description"])
            .stdin(c.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?)
            .output()?
            .stdout
    } else {
        Vec::new()
    };
    let list = String::from_utf8(raw)?
        .split('\n')
        .map(|s| s.trim().replace("device.description = ", "").replace('\"', ""))
        .filter(|s| !s.is_empty())
        .collect();
    Ok(list)
}

/// Compute the output file path from the folder chooser, name entry, and format combobox.
pub fn build_filename(
    folder: &str,
    name: &str,
    format: &str,
) -> String {
    use chrono::Utc;
    let stem = if name.trim().is_empty() {
        Utc::now().format("%d-%m-%Y_%H.%M.%S.%6f").to_string()
    } else {
        name.trim().to_string()
    };
    PathBuf::from(folder).join(format!("{}.{}", stem, format)).to_string_lossy().to_string()
}
