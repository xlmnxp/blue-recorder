use anyhow::{Error, Result};
use gst::prelude::*;
use gstreamer as gst;
use std::collections::HashMap;
use zbus::{
    export::futures_util::TryStreamExt,
    message, proxy,
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
    ) -> (i32, i32) {
        self.screen_cast_proxy
            .create_session(HashMap::from([
                ("handle_token", Value::from("blue_recorder_1")),
                ("session_handle_token", Value::from("blue_recorder_1")),
            ]))
            .await
            .expect("failed to create session");

        let (mut height, mut width) = (0, 0);

        let mut message_stream = MessageStream::from(self.connection.clone());

        self.filename = filename.clone();

        while let Some(msg) = message_stream
            .try_next()
            .await
            .expect("failed to get message")
        {
            match msg.message_type() {
                message::Type::Signal => {
                    let body = msg.body();
                    let (response_num, response): (u32, HashMap<&str, Value>) =
                        body.deserialize().unwrap();

                    if response_num > 0 {
                        return (height, width);
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
                        let stream = self.record_screen_cast(response.clone())
                            .await
                            .expect("failed to record screen cast");

                        width = stream.0;
                        height = stream.1;
                        break;
                    }
                }
                _ => {
                    println!("\n\nUnkown message: {:?}", msg);
                }
            }
        }

        (height, width)
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

    async fn record_screen_cast(&mut self, response: HashMap<&str, Value<'_>>) -> Result<(i32, i32)> {
        let streams: &Value<'_> = response.get("streams").expect("cannot get streams");

        let (mut width, mut height) = (0, 0);

        let stream_fields = streams
            .clone()
            .downcast::<Vec<Value>>()
            .expect("cannot down cast streams to vec array")
            .first()
            .expect("cannot get first object from streams array")
            .clone()
            .downcast::<Structure>()
            .expect("cannot down cast first object to structure");

        if let Some(field) = stream_fields.fields().get(1) {
            let dict = field
                .clone()
                .downcast::<Dict>()
                .expect("cannot down cast field to value");

            let size_str = Str::from("size");
            let size = dict
                .get::<Str, Structure>(&size_str)
                .expect("cannot get size")
                .expect("cannot get size structure");

            let fields = size.fields();

            let size = fields
                .iter()
                .map(|field| {
                    field
                        .clone()
                        .downcast::<i32>()
                        .expect("cannot down cast width to i32")
                })
                .collect::<Vec<i32>>();

            let [stream_width, stream_height] = size.as_slice() else {
                return Err(Error::msg("cannot get width and height"));
            };

            width = *stream_width;
            height = *stream_height;
        }

        // get fields from nested structure inside elements
        // NOTICE: this is not the best way to get node_id, but it works for now
        let stream_node_id: u32 = stream_fields.fields()
            .first()
            .expect("cannot get first field from structure")
            .clone()
            .downcast::<u32>()
            .expect("cannot down cast first field to u32");

        // launch gstreamer pipeline
        let gst_element: gst::Element = gst::parse::launch(&format!(
                "pipewiresrc path={stream_node_id} ! videorate ! video/x-raw,framerate=60/1 ! videoconvert ! vp8enc min-quantizer=0 max-quantizer=1 keyframe-mode=disabled buffer-size=20000 ! webmmux ! filesink location={filename}",
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
        Ok((width, height))
    }
}
