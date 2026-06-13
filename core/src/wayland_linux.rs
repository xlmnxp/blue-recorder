use anyhow::{anyhow, Result};
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

/// Sends EOS and gives the muxer a bounded window to flush and write its
/// trailer before tearing the pipeline down with `Null`. An abrupt `Null`
/// skips matroskamux's finalization and truncates the recording's tail —
/// a race that's normally too fast to lose natively, but snap's extra
/// sandboxing/IPC overhead widens it enough to reproduce reliably.
pub fn shutdown_pipeline(pipeline: &gst::Pipeline) {
    if let Some(bus) = pipeline.bus() {
        let _ = pipeline.send_event(gst::event::Eos::new());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
                match msg.view() {
                    gst::MessageView::Eos(_) | gst::MessageView::Error(_) => break,
                    _ => {}
                }
            }
        }
    }
    let _ = pipeline.set_state(gst::State::Null);
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
    monitor_logical_sizes: Vec<(i32, i32, i32, i32)>,
}

impl WaylandRecorder {
    pub fn is_active(&self) -> bool {
        self.pipeline.is_some()
    }

    pub fn set_monitor_logical_sizes(&mut self, sizes: Vec<(i32, i32, i32, i32)>) {
        self.monitor_logical_sizes = sizes;
    }

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
            monitor_logical_sizes: Vec::new(),
        }
    }

    pub async fn start(
        &mut self,
        filename: String,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
        framerate: u16,
        select_area: bool,
        area_selector: Option<&dyn Fn(i32, i32, i32, i32) -> Option<(u16, u16, u16, u16)>>,
    ) -> (i32, i32) {
        self.filename = filename;

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
                let start_req = self
                    .handle_session(
                        self.screen_cast_proxy.clone(),
                        &mut message_stream,
                        response.clone(),
                        record_type,
                        cursor_mode_type,
                        &tok_select,
                        &tok_start,
                    )
                    .await
                    .expect("failed to handle session");

                our_paths.push(start_req.to_string());
                continue;
            }

            if response.contains_key("streams") {
                let monitor = parse_monitor_geometry(&response);
                // Look up the GDK logical dimensions for this monitor.
                // These match the GDK logical coordinate space the area selector works in.
                let (gdk_lw, gdk_lh) = self.monitor_logical_sizes.iter()
                    .find(|&&(lx, ly, _, _)| lx == monitor.0 && ly == monitor.1)
                    .map(|&(_, _, lw, lh)| (lw, lh))
                    .unwrap_or((monitor.2, monitor.3));
                let crop = if select_area {
                    area_selector.and_then(|f| f(monitor.0, monitor.1, gdk_lw, gdk_lh))
                } else {
                    None
                };
                let (w, h) = self
                    .record_screen_cast(response.clone(), framerate, crop, gdk_lw, gdk_lh)
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
                shutdown_pipeline(&pipeline);
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
        message_stream: &mut MessageStream,
        response: HashMap<&str, Value<'_>>,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
        tok_select: &str,
        tok_start: &str,
    ) -> Result<OwnedObjectPath> {
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

        // The portal completes SelectSources asynchronously — its own Response
        // signal on `select_request` is the only reliable signal that sources
        // are actually selected. Calling Start before that lands races with
        // the portal and can fail with "Sources not selected" (seen mostly
        // under the snap, where the extra D-Bus proxy hop adds enough latency
        // to expose the race).
        wait_for_request_response(message_stream, &select_request).await?;

        let start_request = screen_cast_proxy
            .start(
                ObjectPath::try_from(session_handle)?,
                "",
                HashMap::from([("handle_token", Value::from(tok_start))]),
            )
            .await?;

        Ok(start_request)
    }

    async fn record_screen_cast(
        &mut self,
        response: HashMap<&str, Value<'_>>,
        framerate: u16,
        crop: Option<(u16, u16, u16, u16)>,
        gdk_lw: i32,
        gdk_lh: i32,
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
        let pipeline = select_pipeline(node_id, fps, &self.filename, crop, gdk_lw, gdk_lh);
        pipeline.set_state(gst::State::Playing).expect("failed to start pipeline");
        self.pipeline = Some(pipeline);

        Ok((width, height))
    }
}

/// Blocks until the `org.freedesktop.portal.Request::Response` signal for
/// `request_path` arrives, returning an error if the request failed/was
/// cancelled (non-zero response code) or the stream ended first.
async fn wait_for_request_response(
    message_stream: &mut MessageStream,
    request_path: &OwnedObjectPath,
) -> Result<()> {
    let request_path = request_path.to_string();

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
        if signal_path != request_path {
            continue;
        }

        let body = msg.body();
        let (response_num, _response): (u32, HashMap<&str, Value>) = body.deserialize()?;
        if response_num != 0 {
            return Err(anyhow!("select_sources request failed or was cancelled"));
        }
        return Ok(());
    }

    Err(anyhow!("message stream ended while waiting for select_sources response"))
}

fn parse_monitor_geometry(response: &HashMap<&str, Value<'_>>) -> (i32, i32, i32, i32) {
    let (mut x, mut y, mut w, mut h) = (0i32, 0i32, 0i32, 0i32);
    let Some(streams) = response.get("streams") else { return (x, y, w, h) };
    let Ok(stream_vec) = streams.clone().downcast::<Vec<Value>>() else { return (x, y, w, h) };
    let Some(first) = stream_vec.into_iter().next() else { return (x, y, w, h) };
    let Ok(structure) = first.downcast::<Structure>() else { return (x, y, w, h) };
    let fields = structure.fields();
    let Some(props_value) = fields.get(1) else { return (x, y, w, h) };
    let Ok(dict) = props_value.clone().downcast::<Dict>() else { return (x, y, w, h) };

    if let Ok(Some(s)) = dict.get::<Str, Structure>(&Str::from("size")) {
        let dims: Vec<i32> = s.fields().iter().filter_map(|f| f.clone().downcast::<i32>().ok()).collect();
        if dims.len() >= 2 { w = dims[0]; h = dims[1]; }
    }
    if let Ok(Some(s)) = dict.get::<Str, Structure>(&Str::from("position")) {
        let dims: Vec<i32> = s.fields().iter().filter_map(|f| f.clone().downcast::<i32>().ok()).collect();
        if dims.len() >= 2 { x = dims[0]; y = dims[1]; }
    }
    (x, y, w, h)
}

/// Probing spins up a real test pipeline and waits up to 600ms for it to play
/// through, so memoize the result per element — the available hardware
/// encoders don't change over the process lifetime, and re-probing on every
/// recording start added a noticeable delay after area selection.
fn probe_encoder(element_name: &str) -> bool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(&cached) = cache.lock().unwrap().get(element_name) {
        return cached;
    }

    let result = probe_encoder_uncached(element_name);
    cache.lock().unwrap().insert(element_name.to_string(), result);
    result
}

fn probe_encoder_uncached(element_name: &str) -> bool {
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

fn select_pipeline(
    node_id: u32,
    fps: u16,
    filename: &str,
    crop: Option<(u16, u16, u16, u16)>,
    logical_w: i32,
    logical_h: i32,
) -> gst::Pipeline {
    let fps_cap = if fps > 0 { fps } else { 30 };
    let crop_element = if crop.is_some() {
        "! videocrop name=vcrop left=0 top=0 right=0 bottom=0 "
    } else {
        ""
    };
    let src = format!(
        "pipewiresrc path={node_id} do-timestamp=true min-buffers=1 max-buffers=8 \
         ! videorate drop-only=true \
         ! video/x-raw,framerate={fps_cap}/1 \
         {crop_element}\
         ! queue leaky=downstream max-size-buffers=2 max-size-time=0 max-size-bytes=0",
    );
    let vp9_opts = format!(
        "deadline=1 cpu-used=5 lag-in-frames=0 end-usage=cbr \
         target-bitrate=8000000 error-resilient=1 threads=4 keyframe-max-dist={fps}",
    );
    let candidates: &[(&str, String)] = &[
        (
            "vaapih264enc",
            format!(
                "{src} ! videoconvert ! vaapipostproc \
                 ! vaapih264enc rate-control=cbr bitrate=8000 \
                 ! h264parse ! matroskamux ! filesink location={filename}",
            ),
        ),
        (
            "nvh264enc",
            format!(
                "{src} ! videoconvert ! nvh264enc \
                 ! h264parse ! matroskamux ! filesink location={filename}",
            ),
        ),
        (
            "vp9enc",
            format!(
                "{src} ! videoconvert n-threads=4 \
                 ! vp9enc {vp9_opts} \
                 ! matroskamux ! filesink location={filename}",
            ),
        ),
    ];

    for (element_name, desc) in candidates {
        if !probe_encoder(element_name) {
            continue;
        }
        let elem = gst::parse::launch(desc).expect("failed to build recording pipeline");
        let pipeline = elem.dynamic_cast::<gst::Pipeline>().expect("not a pipeline");

        if let Some((lcx, lcy, lcw, lch)) = crop {
            if logical_w > 0 && logical_h > 0 {
                if let Some(vcrop) = pipeline.by_name("vcrop") {
                    if let Some(sink_pad) = vcrop.static_pad("sink") {
                        let vcrop_clone = vcrop.clone();
                        let lw = logical_w;
                        let lh = logical_h;
                        sink_pad.add_probe(
                            gst::PadProbeType::EVENT_DOWNSTREAM,
                            move |_, info| {
                                let Some(gst::PadProbeData::Event(ref ev)) = info.data else {
                                    return gst::PadProbeReturn::Ok;
                                };
                                if ev.type_() != gst::EventType::Caps {
                                    return gst::PadProbeReturn::Ok;
                                }
                                let gst::EventView::Caps(caps_ev) = ev.view() else {
                                    return gst::PadProbeReturn::Ok;
                                };
                                let caps = caps_ev.caps();
                                let Some(s) = caps.structure(0) else {
                                    return gst::PadProbeReturn::Remove;
                                };
                                let (Ok(aw), Ok(ah)) = (s.get::<i32>("width"), s.get::<i32>("height")) else {
                                    return gst::PadProbeReturn::Remove;
                                };
                                let sx = aw as f64 / lw as f64;
                                let sy = ah as f64 / lh as f64;
                                let left   = (lcx as f64 * sx).round() as u32;
                                let top    = (lcy as f64 * sy).round() as u32;
                                let cw_p   = (lcw as f64 * sx).round().max(1.0) as u32;
                                let ch_p   = (lch as f64 * sy).round().max(1.0) as u32;
                                let right  = (aw as u32).saturating_sub(left + cw_p);
                                let bottom = (ah as u32).saturating_sub(top + ch_p);
                                vcrop_clone.set_property("left",   left   as i32);
                                vcrop_clone.set_property("top",    top    as i32);
                                vcrop_clone.set_property("right",  right  as i32);
                                vcrop_clone.set_property("bottom", bottom as i32);
                                gst::PadProbeReturn::Remove
                            },
                        );
                    }
                }
            }
        }

        return pipeline;
    }

    panic!("No working GStreamer video encoder found. \
            Install gst-plugins-good for VP9 support.");
}
