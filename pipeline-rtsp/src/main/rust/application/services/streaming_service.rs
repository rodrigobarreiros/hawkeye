use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::entities::StreamSession;
use crate::domain::errors::Result;
use crate::domain::ports::{MetricsReporter, StreamingServer};
use crate::domain::value_objects::{ServerConfig, StreamConfig};

/// Application service orchestrating streaming operations
pub struct StreamingService {
    server: Arc<RwLock<Box<dyn StreamingServer>>>,
    metrics: Arc<dyn MetricsReporter>,
}

impl StreamingService {
    pub fn new(server: Box<dyn StreamingServer>, metrics: Arc<dyn MetricsReporter>) -> Self {
        Self {
            server: Arc::new(RwLock::new(server)),
            metrics,
        }
    }

    /// Start streaming session (use case)
    pub async fn start_streaming(
        &self,
        stream_config: StreamConfig,
        server_config: ServerConfig,
    ) -> Result<StreamSession> {
        // Validate stream configuration
        stream_config.validate()?;

        // Start server
        let session = {
            let mut server = self.server.write().await;
            server.start(stream_config, server_config).await?
        };

        // Report metrics
        self.metrics.report_session_started(&session);

        tracing::info!(
            session_id = %session.id(),
            mount_point = %session.server_config().mount_point(),
            "Streaming session started"
        );

        Ok(session)
    }

    /// Stop streaming session
    pub async fn stop_streaming(&self) -> Result<()> {
        let mut server = self.server.write().await;

        if !server.is_running() {
            return Ok(());
        }

        // Get session before stopping for metrics
        if let Some(session) = server.current_session() {
            self.metrics.report_session_stopped(session);
        }

        tracing::info!("Stopping streaming session");
        server.stop().await?;

        Ok(())
    }

    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        let server = self.server.read().await;
        server.is_running()
    }

    /// Get current session info
    pub async fn current_session(&self) -> Option<StreamSession> {
        let server = self.server.read().await;
        server.current_session().cloned()
    }
}
