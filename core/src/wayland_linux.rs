use anyhow::Result;
use futures_util::StreamExt;
use gst::prelude::*;
use gstreamer as gst;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use zbus::{
    message,
    proxy,
    zvariant::{Dict, ObjectPath, OwnedObjectPath, Str, Structure, Value},
    Connection, MessageStream,
};

#[derive(Clone, Copy)]
pub enum RecordTypes {
    Default,
    Monitor,
    Window,
    MonitorOrWindow,
}

#[derive(Clone, Copy)]
pub enum CursorModeTypes {
    Default,
    Hidden,
    Show,
}

#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenCast {
    async fn create_session(&self, options: HashMap<&str, Value<'_>>) -> Result<OwnedObjectPath>;
    async fn select_sources(
        &self,
        session_handle: ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
    ) -> Result<OwnedObjectPath>;
    async fn start(
        &self,
        session_handle: ObjectPath<'_>,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> Result<OwnedObjectPath>;
}

#[derive(Clone)]
pub struct WaylandRecorder {
    connection: Connection,
    screen_cast_proxy: ScreenCastProxy<'static>,
    session_path: String,
    pipeline: Option<gst::Pipeline>,
    filename: String,
}

impl WaylandRecorder {
    /// Returns true if the GStreamer pipeline is running (recording is active).
    pub fn is_active(&self) -> bool {
        self.pipeline.is_some()
    }

    /// Extract the data needed to stop recording off the main thread.
    /// Takes ownership of the pipeline and session info, leaving the recorder
    /// in a clean idle state.
    pub fn take_for_stop(&mut self) -> (Option<gst::Pipeline>, zbus::Connection, String) {
        let pipeline     = self.pipeline.take();
        let connection   = self.connection.clone();
        let session_path = std::mem::take(&mut self.session_path);
        (pipeline, connection, session_path)
    }

    pub async fn new() -> Self {
        let connection = Connection::session()
            .await
            .expect("failed to connect to session bus");
        let screen_cast_proxy = ScreenCastProxy::new(&connection)
            .await
            .expect("failed to create dbus proxy for screen-cast");
        gst::init().expect("failed to initialize gstreamer");

        WaylandRecorder {
            connection,
            screen_cast_proxy,
            session_path: String::new(),
            filename: String::from("blue_recorder.mkv"),
            pipeline: None,
        }
    }

    pub async fn start(
        &mut self,
        filename: String,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
        framerate: u16,
    ) -> (i32, i32) {
        self.filename = filename;

        // Use a session-unique counter so every recording session gets different
        // request paths. Fixed tokens caused stale Response signals from the
        // previous session (same path) to be processed immediately, making the
        // portal picker appear to be skipped and producing bad recordings.
        static SESSION: AtomicU32 = AtomicU32::new(0);
        let sid = SESSION.fetch_add(1, Ordering::Relaxed);
        let tok_session = format!("br_s{}_sess", sid);
        let tok_select  = format!("br_s{}_sel",  sid);
        let tok_start   = format!("br_s{}_start", sid);

        let create_request = self.screen_cast_proxy
            .create_session(HashMap::from([
                ("handle_token",         Value::from(tok_session.as_str())),
                ("session_handle_token", Value::from(tok_session.as_str())),
            ]))
            .await
            .expect("failed to create session");

        let (mut width, mut height) = (0i32, 0i32);
        let mut message_stream = MessageStream::from(self.connection.clone());
        let mut our_paths: Vec<String> = vec![create_request.to_string()];

        while let Some(msg) = message_stream.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(_) => continue,
            };

            if msg.message_type() != message::Type::Signal {
                continue;
            }

            let header = msg.header();
            let is_response = header
                .interface()
                .map(|i| i.as_str() == "org.freedesktop.portal.Request")
                .unwrap_or(false)
                && header
                    .member()
                    .map(|m| m.as_str() == "Response")
                    .unwrap_or(false);

            if !is_response {
                continue;
            }

            let signal_path = header
                .path()
                .map(|p| p.to_string())
                .unwrap_or_default();
            if !our_paths.contains(&signal_path) {
                continue;
            }

            let body = msg.body();
            let (response_num, response): (u32, HashMap<&str, Value>) =
                match body.deserialize() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

            if response_num > 0 {
                return (width, height);
            }

            if response.is_empty() {
                continue;
            }

            if response.contains_key("session_handle") {
                let (select_req, start_req) = self
                    .handle_session(
                        self.screen_cast_proxy.clone(),
                        response.clone(),
                        record_type,
                        cursor_mode_type,
                        &tok_select,
                        &tok_start,
                    )
                    .await
                    .expect("failed to handle session");

                our_paths.push(select_req.to_string());
                our_paths.push(start_req.to_string());
                continue;
            }

            if response.contains_key("streams") {
                let (w, h) = self
                    .record_screen_cast(response.clone(), framerate)
                    .await
                    .expect("failed to record screen cast");
                width = w;
                height = h;
                break;
            }
        }

        (width, height)
    }

    pub async fn stop(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            // Run blocking set_state(Null) on a background thread so the async
            // executor stays free.  We poll a flag via the async runtime so
            // the caller (glib::MainContext::block_on) keeps processing events.
            use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
            let done  = Arc::new(AtomicBool::new(false));
            let done2 = done.clone();
            std::thread::spawn(move || {
                let _ = pipeline.set_state(gst::State::Null);
                done2.store(true, Ordering::Release);
            });
            // async_std::task::sleep yields to the executor each tick so the
            // GLib main loop can repaint while GStreamer flushes.
            while !done.load(Ordering::Acquire) {
                async_std::task::sleep(std::time::Duration::from_millis(16)).await;
            }
        }

        if !self.session_path.is_empty() {
            let _ = self
                .connection
                .call_method(
                    Some("org.freedesktop.portal.Desktop"),
                    self.session_path.as_str(),
                    Some("org.freedesktop.portal.Session"),
                    "Close",
                    &(),
                )
                .await;
            self.session_path.clear();
        }
    }

    async fn handle_session(
        &mut self,
        screen_cast_proxy: ScreenCastProxy<'_>,
        response: HashMap<&str, Value<'_>>,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
        tok_select: &str,
        tok_start: &str,
    ) -> Result<(OwnedObjectPath, OwnedObjectPath)> {
        let session_handle = response
            .get("session_handle")
            .expect("missing session_handle")
            .clone()
            .downcast::<String>()
            .expect("session_handle is not a string");

        self.session_path = session_handle.clone();

        let select_request = screen_cast_proxy
            .select_sources(
                ObjectPath::try_from(session_handle.clone())?,
                HashMap::from([
                    ("handle_token", Value::from(tok_select)),
                    (
                        "types",
                        Value::from(match record_type {
                            RecordTypes::Monitor => 1u32,
                            RecordTypes::Window => 2u32,
                            RecordTypes::MonitorOrWindow => 3u32,
                            RecordTypes::Default => 1u32,
                        }),
                    ),
                    (
                        "cursor_mode",
                        Value::from(match cursor_mode_type {
                            CursorModeTypes::Hidden => 1u32,
                            CursorModeTypes::Show => 2u32,
                            CursorModeTypes::Default => 2u32,
                        }),
                    ),
                    ("multiple", Value::from(false)),
                ]),
            )
            .await?;

        let start_request = screen_cast_proxy
            .start(
                ObjectPath::try_from(session_handle)?,
                "",
                HashMap::from([("handle_token", Value::from(tok_start))]),
            )
            .await?;

        Ok((select_request, start_request))
    }

    async fn record_screen_cast(
        &mut self,
        response: HashMap<&str, Value<'_>>,
        framerate: u16,
    ) -> Result<(i32, i32)> {
        let streams = response.get("streams").expect("missing streams");

        let (mut width, mut height) = (0i32, 0i32);

        let stream_vec = streams
            .clone()
            .downcast::<Vec<Value>>()
            .expect("streams is not an array");

        let first = stream_vec
            .into_iter()
            .next()
            .expect("streams array is empty")
            .downcast::<Structure>()
            .expect("stream entry is not a structure");

        let fields = first.fields();

        let node_id: u32 = fields
            .first()
            .expect("missing node_id field")
            .clone()
            .downcast::<u32>()
            .expect("node_id is not u32");

        if let Some(props_value) = fields.get(1) {
            if let Ok(dict) = props_value.clone().downcast::<Dict>() {
                let size_key = Str::from("size");
                if let Ok(Some(size_struct)) = dict.get::<Str, Structure>(&size_key) {
                    let dims: Vec<i32> = size_struct
                        .fields()
                        .iter()
                        .filter_map(|f| f.clone().downcast::<i32>().ok())
                        .collect();
                    if dims.len() >= 2 {
                        width = dims[0];
                        height = dims[1];
                    }
                }
            }
        }

        let fps = if framerate > 0 { framerate } else { 30 };

        let pipeline = select_pipeline(node_id, fps, &self.filename);
        pipeline.set_state(gst::State::Playing).expect("failed to start pipeline");
        self.pipeline = Some(pipeline);

        Ok((width, height))
    }
}

// ---------------------------------------------------------------------------
// Encoder selection
// ---------------------------------------------------------------------------

/// Probe an encoder with a synthetic video source and a fakesink so no real
/// files are created and no PipeWire connection is needed.
/// Returns true if the encoder initialises and produces frames without errors.
fn probe_encoder(element_name: &str) -> bool {
    if gst::ElementFactory::find(element_name).is_none() {
        return false;
    }
    let encoder_chain = match element_name {
        "vaapih264enc" => "videoconvert ! vaapipostproc ! vaapih264enc ! fakesink",
        "nvh264enc"    => "videoconvert ! nvh264enc ! fakesink",
        "vp9enc"       => "videoconvert ! vp9enc ! fakesink",
        _              => return false,
    };
    let desc = format!("videotestsrc num-buffers=10 ! {}", encoder_chain);
    let Ok(elem) = gst::parse::launch(&desc) else { return false };
    let Ok(pipeline) = elem.dynamic_cast::<gst::Pipeline>() else { return false };

    if pipeline.set_state(gst::State::Playing).is_err() {
        let _ = pipeline.set_state(gst::State::Null);
        return false;
    }

    let bus = pipeline.bus().expect("pipeline has no bus");
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(600);
    let mut ok = true;
    while std::time::Instant::now() < deadline {
        match bus.timed_pop(gst::ClockTime::from_mseconds(50)) {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(_) => { ok = false; break; }
                gst::MessageView::Eos(_)   => break,
                _ => {}
            },
            None => break,
        }
    }
    let _ = pipeline.set_state(gst::State::Null);
    ok
}

/// Build and return the capture pipeline for the first working encoder.
/// Probing uses a synthetic source + fakesink so no output file is touched
/// until recording actually begins.
///
/// All pipelines use matroskamux (.mkv) so ffmpeg can handle them uniformly
/// regardless of the codec chosen.
fn select_pipeline(node_id: u32, fps: u16, filename: &str) -> gst::Pipeline {
    let fps_cap = if fps > 0 { fps } else { 30 };
    // video/x-raw,max-framerate is NOT a standard GStreamer caps field and is
    // silently ignored — PipeWire would keep delivering at 60 fps and the
    // encoder would declare 60 fps in the stream header.
    //
    // Instead: videorate drop-only=true drops excess frames without duplicating
    // (so no artificial latency), then video/x-raw,framerate=N/1 negotiates
    // the exact rate downstream so the encoder and muxer declare it correctly.
    let src = format!(
        "pipewiresrc path={node_id} do-timestamp=true \
         ! videorate drop-only=true \
         ! video/x-raw,framerate={fps_cap}/1 \
         ! queue leaky=downstream max-size-buffers=2 max-size-time=0 max-size-bytes=0",
        node_id = node_id,
        fps_cap = fps_cap,
    );

    // VP9 quality: cpu-used=5 balances real-time speed and quality (0=best, 9=fastest).
    // end-usage=cbr + target-bitrate=8000000 gives ~8 Mbps — good for 1080p.
    let vp9_opts = format!(
        "deadline=1 cpu-used=5 lag-in-frames=0 end-usage=cbr \
         target-bitrate=8000000 error-resilient=1 threads=4 keyframe-max-dist={fps}",
        fps = fps,
    );

    let candidates: &[(&str, String)] = &[
        (
            "vaapih264enc",
            format!(
                "{src} ! videoconvert ! vaapipostproc \
                 ! vaapih264enc rate-control=cbr bitrate=8000 \
                 ! h264parse ! matroskamux ! filesink location={f}",
                src = src, f = filename,
            ),
        ),
        (
            "nvh264enc",
            format!(
                "{src} ! videoconvert ! nvh264enc \
                 ! h264parse ! matroskamux ! filesink location={f}",
                src = src, f = filename,
            ),
        ),
        (
            "vp9enc",
            format!(
                "{src} ! videoconvert n-threads=4 \
                 ! vp9enc {opts} \
                 ! matroskamux ! filesink location={f}",
                src = src, opts = vp9_opts, f = filename,
            ),
        ),
    ];

    for (element_name, desc) in candidates {
        if !probe_encoder(element_name) {
            continue;
        }
        let elem = gst::parse::launch(desc).expect("failed to build recording pipeline");
        let pipeline = elem.dynamic_cast::<gst::Pipeline>().expect("not a pipeline");
        return pipeline;
    }

    panic!("No working GStreamer video encoder found. \
            Install gst-plugins-good for VP9 support.");
}
