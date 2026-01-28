use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::domain::errors::Result;
use crate::domain::value_objects::BridgeConfig;

/// Port for stream bridge implementations
pub trait StreamBridge: Send {
    /// Run the bridge once until completion or error
    /// The running flag can be used to signal graceful shutdown
    fn run_once(&mut self) -> Result<()>;

    /// Run the bridge with a shutdown signal
    /// Implementations should periodically check this flag and exit gracefully
    fn run_once_with_shutdown(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        // Default implementation ignores shutdown signal for backwards compatibility
        let _ = running;
        self.run_once()
    }

    /// Get the bridge configuration
    fn config(&self) -> &BridgeConfig;
}
