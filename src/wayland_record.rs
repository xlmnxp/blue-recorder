use gst::prelude::*;
use gstreamer as gst;
use std::collections::HashMap;
use zbus::{
    dbus_proxy,
    export::futures_util::TryStreamExt,
    zvariant::{ObjectPath, OwnedObjectPath, Structure, Value},
    Connection, MessageStream, MessageType, Result,
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

#[dbus_proxy(
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
            filename: String::from("blue_recorder.webm"),
            pipeline: None,
        }
    }

    pub async fn start(
        &mut self,
        filename: String,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
    ) -> bool {
        self.screen_cast_proxy
            .create_session(HashMap::from([
                ("handle_token", Value::from("blue_recorder_1")),
                ("session_handle_token", Value::from("blue_recorder_1")),
            ]))
            .await
            .expect("failed to create session");

        let mut message_stream = MessageStream::from(self.connection.clone());

        self.filename = filename.clone();

        while let Some(msg) = message_stream
            .try_next()
            .await
            .expect("failed to get message")
        {
            match msg.message_type() {
                MessageType::Signal => {
                    let (response_num, response) = msg
                        .body::<(u32, HashMap<&str, Value>)>()
                        .expect("failed to get body");

                    if response_num > 0 {
                        return false;
                    }

                    if response.len() == 0 {
                        continue;
                    }

                    if response.contains_key("session_handle") {
                        self.handle_session(
                            self.screen_cast_proxy.clone(),
                            response.clone(),
                            record_type,
                            cursor_mode_type,
                        )
                        .await
                        .expect("failed to handle session");
                        continue;
                    }

                    if response.contains_key("streams") {
                        self.record_screen_cast(response.clone())
                            .await
                            .expect("failed to record screen cast");
                        break;
                    }
                }
                _ => {
                    println!("\n\nUnkown message: {:?}", msg);
                }
            }
        }

        true
    }

    pub async fn stop(&mut self) {
        if let Some(pipeline) = self.pipeline.clone() {
            pipeline
                .set_state(gst::State::Null)
                .expect("failed to stop pipeline");
        }

        if self.session_path.len() > 0 {
            println!(
                "Closing session...: {:?}",
                self.session_path.replace("request", "session")
            );
            self.connection
                .clone()
                .call_method(
                    Some("org.freedesktop.portal.Desktop"),
                    self.session_path.clone().replace("request", "session"),
                    Some("org.freedesktop.portal.Session"),
                    "Close",
                    &(),
                )
                .await
                .expect("failed to close session");
            self.session_path = String::new();
        }
    }

    async fn handle_session(
        &mut self,
        screen_cast_proxy: ScreenCastProxy<'_>,
        response: HashMap<&str, Value<'_>>,
        record_type: RecordTypes,
        cursor_mode_type: CursorModeTypes,
    ) -> Result<()> {
        let response_session_handle = response
            .get("session_handle")
            .expect("cannot get session_handle")
            .clone()
            .downcast::<String>()
            .expect("cannot down cast session_handle");

        self.session_path = response_session_handle.clone();

        screen_cast_proxy
            .select_sources(
                ObjectPath::try_from(response_session_handle.clone())?,
                HashMap::from([
                    ("handle_token", Value::from("blue_recorder_1")),
                    (
                        "types",
                        Value::from(match record_type {
                            RecordTypes::Monitor => 1u32,
                            RecordTypes::Window => 2u32,
                            RecordTypes::MonitorOrWindow => 3u32,
                            _ => 0u32,
                        }),
                    ),
                    (
                        "cursor_mode",
                        Value::from(match cursor_mode_type {
                            CursorModeTypes::Hidden => 1u32,
                            CursorModeTypes::Show => 2u32,
                            _ => 0u32,
                        }),
                    ),
                ]),
            )
            .await?;

        screen_cast_proxy
            .start(
                ObjectPath::try_from(response_session_handle.clone())?,
                "parent_window",
                HashMap::from([("handle_token", Value::from("blue_recorder_1"))]),
            )
            .await?;
        Ok(())
    }

    async fn record_screen_cast(&mut self, response: HashMap<&str, Value<'_>>) -> Result<()> {
        let streams: &Value<'_> = response.get("streams").expect("cannot get streams");

        // get fields from nested structure inside elements
        // NOTICE: this is not the best way to get node_id, but it works for now
        let stream_node_id: u32 = streams
            .clone()
            .downcast::<Vec<Value>>()
            .expect("cannot down cast streams to vec array")
            .get(0)
            .expect("cannot get first object from streams array")
            .clone()
            .downcast::<Structure>()
            .expect("cannot down cast first object to structure")
            .fields()
            .get(0)
            .expect("cannot get first field from structure")
            .clone()
            .downcast::<u32>()
            .expect("cannot down cast first field to u32");

        // launch gstreamer pipeline
        let gst_element: gst::Element = gst::parse_launch(&format!(
                "pipewiresrc path={stream_node_id} ! videorate ! video/x-raw,framerate=30/1 ! videoconvert chroma-mode=none dither=none matrix-mode=output-only ! vp8enc max-quantizer=17 deadline=1 keyframe-mode=disabled buffer-size=20000 ! webmmux ! filesink location={filename}",
                filename = self.filename
            )).expect("failed to launch gstreamer pipeline");

        // start pipeline
        let pipeline: gst::Pipeline = gst_element
            .dynamic_cast::<gst::Pipeline>()
            .expect("pipeline error");

        self.pipeline = Some(pipeline.clone());

        pipeline
            .set_state(gst::State::Playing)
            .expect("failed to start pipeline");

        println!("Recording Wayland screen cast...");
        Ok(())
    }
}
