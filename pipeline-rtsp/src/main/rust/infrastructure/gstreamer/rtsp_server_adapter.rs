use async_trait::async_trait;
use gstreamer::prelude::*;
use gstreamer_rtsp_server as gst_rtsp;
use gstreamer_rtsp_server::prelude::*;

use super::PipelineBuilder;
use crate::domain::entities::StreamSession;
use crate::domain::errors::{DomainError, Result};
use crate::domain::ports::StreamingServer;
use crate::domain::value_objects::{ServerConfig, StreamConfig};

pub struct GStreamerRtspServer {
    server: Option<gst_rtsp::RTSPServer>,
    current_session: Option<StreamSession>,
    #[allow(dead_code)]
    server_id: Option<glib::SourceId>,
}

impl GStreamerRtspServer {
    pub fn new() -> Self {
        Self {
            server: None,
            current_session: None,
            server_id: None,
        }
    }

    fn setup_looping(factory: &gst_rtsp::RTSPMediaFactory, enabled: bool) {
        if !enabled {
            return;
        }

        factory.connect_media_configure(|_factory, media| {
            let element = media.element();
            if let Some(bus) = element.bus() {
                let element_weak = element.downgrade();
                let _ = bus.add_watch(move |_bus, msg: &gstreamer::Message| {
                    use gstreamer::MessageView;

                    if let Some(element) = element_weak.upgrade() {
                        match msg.view() {
                            MessageView::Eos(..) => {
                                let _ = element.seek_simple(
                                    gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                                    gstreamer::ClockTime::ZERO,
                                );
                            }
                            MessageView::Error(err) => {
                                tracing::error!("Pipeline error: {:?}", err);
                            }
                            _ => {}
                        }
                    }

                    glib::ControlFlow::Continue
                });
            }
        });
    }
}

impl Default for GStreamerRtspServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamingServer for GStreamerRtspServer {
    async fn start(
        &mut self,
        stream_config: StreamConfig,
        server_config: ServerConfig,
    ) -> Result<StreamSession> {
        // Create GStreamer server
        let server = gst_rtsp::RTSPServer::new();
        server.set_service(&server_config.port().to_string());

        // Get mount points
        let mounts = server.mount_points().ok_or(DomainError::ServerInitFailed)?;

        // Create media factory
        let factory = gst_rtsp::RTSPMediaFactory::new();

        // Build pipeline from domain config
        let pipeline_str = PipelineBuilder::build_launch_string(&stream_config);
        factory.set_launch(&pipeline_str);
        factory.set_shared(true);
        factory.set_eos_shutdown(false);

        // Setup looping if enabled
        Self::setup_looping(&factory, server_config.looping_enabled());

        // Mount factory
        mounts.add_factory(server_config.mount_point(), factory);

        // Attach server to main context to start listening
        let server_id = server
            .attach(None)
            .map_err(|_| DomainError::ServerInitFailed)?;

        // Create session
        let mut session = StreamSession::new(stream_config, server_config);
        session.activate();

        self.server = Some(server);
        self.server_id = Some(server_id);
        self.current_session = Some(session.clone());

        Ok(session)
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            session.stop();
        }

        // Server will be dropped and cleaned up
        self.server = None;

        if let Some(session) = &mut self.current_session {
            session.mark_stopped();
        }
        self.current_session = None;

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.server.is_some()
    }

    fn current_session(&self) -> Option<&StreamSession> {
        self.current_session.as_ref()
    }
}
