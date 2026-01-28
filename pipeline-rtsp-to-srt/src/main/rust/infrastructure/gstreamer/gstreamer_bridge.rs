use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Context;
use gstreamer::prelude::*;

use super::PipelineBuilder;
use crate::domain::errors::{DomainError, Result};
use crate::domain::ports::StreamBridge;
use crate::domain::value_objects::BridgeConfig;

/// Timeout for bus polling (100ms allows responsive shutdown)
const BUS_POLL_TIMEOUT_MS: u64 = 100;

pub struct GStreamerBridge {
    config: BridgeConfig,
}

impl GStreamerBridge {
    pub fn new(config: BridgeConfig) -> Self {
        Self { config }
    }

    fn create_pipeline(&self) -> anyhow::Result<gstreamer::Pipeline> {
        let pipeline_str = PipelineBuilder::build_pipeline_string(&self.config);
        tracing::info!("Creating pipeline: {}", pipeline_str);

        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .context("Failed to parse pipeline")?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| anyhow::anyhow!("Failed to downcast to Pipeline"))?;

        Ok(pipeline)
    }

    fn process_bus_message(
        msg: &gstreamer::Message,
        pipeline: &gstreamer::Pipeline,
    ) -> std::result::Result<bool, DomainError> {
        match msg.view() {
            gstreamer::MessageView::Eos(_) => {
                tracing::info!("End of stream");
                Ok(true) // Signal to stop processing
            }
            gstreamer::MessageView::Error(err) => {
                let error_msg = format!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                let _ = pipeline.set_state(gstreamer::State::Null);
                Err(DomainError::PipelineExecutionFailed(error_msg))
            }
            gstreamer::MessageView::StateChanged(state_changed) => {
                if state_changed
                    .src()
                    .map(|s| s == pipeline)
                    .unwrap_or(false)
                {
                    tracing::debug!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_changed.old(),
                        state_changed.current()
                    );
                }
                Ok(false) // Continue processing
            }
            gstreamer::MessageView::Warning(warn) => {
                tracing::warn!(
                    "Warning from {:?}: {} ({:?})",
                    warn.src().map(|s| s.path_string()),
                    warn.error(),
                    warn.debug()
                );
                Ok(false) // Continue processing
            }
            _ => Ok(false), // Continue processing
        }
    }
}

impl StreamBridge for GStreamerBridge {
    fn run_once(&mut self) -> Result<()> {
        // Use a dummy running flag that's always true for backwards compatibility
        let running = Arc::new(AtomicBool::new(true));
        self.run_once_with_shutdown(running)
    }

    fn run_once_with_shutdown(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        let pipeline = self
            .create_pipeline()
            .map_err(|e| DomainError::PipelineCreationFailed(e.to_string()))?;

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| DomainError::PipelineExecutionFailed(e.to_string()))?;

        let bus = pipeline
            .bus()
            .ok_or_else(|| DomainError::PipelineExecutionFailed("Failed to get bus".to_string()))?;

        // Use a timed iterator to allow periodic shutdown checks
        let timeout = gstreamer::ClockTime::from_mseconds(BUS_POLL_TIMEOUT_MS);

        loop {
            // Check shutdown signal before processing
            if !running.load(Ordering::SeqCst) {
                tracing::info!("Shutdown signal received, stopping pipeline");
                break;
            }

            // Poll for messages with timeout
            if let Some(msg) = bus.timed_pop(timeout) {
                match Self::process_bus_message(&msg, &pipeline) {
                    Ok(true) => break,  // EOS received
                    Ok(false) => {}     // Continue processing
                    Err(e) => return Err(e),
                }
            }
            // Timeout expired without message - loop continues to check shutdown
        }

        let _ = pipeline.set_state(gstreamer::State::Null);
        Ok(())
    }

    fn config(&self) -> &BridgeConfig {
        &self.config
    }
}
