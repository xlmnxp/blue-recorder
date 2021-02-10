use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::thread::sleep;

pub struct Ffmpeg {
    pub filename: PathBuf,
    pub record_video: bool,
    pub record_audio: bool,
    pub audio_id: String,
    pub record_mouse: bool,
    pub follow_mouse: bool,
    pub record_frames: String,
    pub record_delay: u64,
}
impl Ffmpeg {
    pub fn record(self, x: i16, y: i16, width: i16, height: i16) -> u32 {
        let mut ffmpeg_command: Command = Command::new("ffmpeg");

        // if recorder video switch is enabled, record video with specified width and hight
        if self.record_video {
            ffmpeg_command.arg("-video_size");
            ffmpeg_command.arg(format!("{}x{}", width, height));
        }

        // if show mouse switch is enabled, draw the mouse to video
        if self.record_mouse {
            ffmpeg_command.arg("-draw_mouse");
            ffmpeg_command.arg("1");
        }

        // if follow mouse switch is enabled, follow the mouse
        if self.follow_mouse {
            ffmpeg_command.arg("-follow_mouse");
            ffmpeg_command.arg("centered");
        }

        ffmpeg_command.arg("-framerate");
        ffmpeg_command.arg(self.record_frames);
        ffmpeg_command.arg("-f");
        ffmpeg_command.arg("x11grab");
        ffmpeg_command.arg("-i");
        ffmpeg_command.arg(format!("{}+{},{}", std::env::var("DISPLAY").expect(":1").as_str(), x, y));

        // if follow audio switch is enabled, record the audio
        if self.record_audio {
            ffmpeg_command.arg("-f");
            ffmpeg_command.arg("pulse");
            ffmpeg_command.arg("-i");
            ffmpeg_command.arg(self.audio_id);
            ffmpeg_command.arg("-strict");
            ffmpeg_command.arg("-2");
        }
        
        ffmpeg_command.arg("-q");
        ffmpeg_command.arg("1");
        ffmpeg_command.arg(self.filename);
        ffmpeg_command.arg("-y");

        // sleep for delay
        sleep(Duration::new(self.record_delay, 0));

        // start recording and return the process id
        ffmpeg_command.spawn().unwrap().id()
    }
}
