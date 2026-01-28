use async_trait::async_trait;

use crate::domain::entities::StreamSession;
use crate::domain::errors::Result;
use crate::domain::value_objects::{ServerConfig, StreamConfig};

/// Port for streaming server implementations
#[async_trait]
pub trait StreamingServer: Send + Sync {
    /// Start server and begin streaming
    async fn start(
        &mut self,
        stream_config: StreamConfig,
        server_config: ServerConfig,
    ) -> Result<StreamSession>;

    /// Stop server gracefully
    async fn stop(&mut self) -> Result<()>;

    /// Check if server is running
    fn is_running(&self) -> bool;

    /// Get current session if any
    fn current_session(&self) -> Option<&StreamSession>;
}
