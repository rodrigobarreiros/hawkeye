use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::domain::entities::ConnectionLifecycle;
use crate::domain::errors::Result;
use crate::domain::ports::{MetricsReporter, StreamBridge};
use crate::domain::value_objects::{BackoffPolicy, ConnectionState};

/// Application service orchestrating the SRT bridge
pub struct BridgeService {
    bridge: Box<dyn StreamBridge>,
    lifecycle: ConnectionLifecycle,
    backoff_policy: BackoffPolicy,
    metrics: Arc<dyn MetricsReporter>,
    running: Arc<AtomicBool>,
}

impl BridgeService {
    pub fn new(
        bridge: Box<dyn StreamBridge>,
        backoff_policy: BackoffPolicy,
        metrics: Arc<dyn MetricsReporter>,
    ) -> Self {
        Self {
            bridge,
            lifecycle: ConnectionLifecycle::new(),
            backoff_policy,
            metrics,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    pub fn current_state(&self) -> ConnectionState {
        *self.lifecycle.current_state()
    }

    /// Run the bridge with automatic reconnection
    pub fn run_with_reconnect(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        let mut current_backoff = self.backoff_policy.initial_delay();
        let mut reconnect_attempt = 0u32;

        // Initial state
        self.lifecycle.transition_to_connecting();
        self.metrics.report_state_change(self.lifecycle.current_state());

        while self.running.load(Ordering::SeqCst) {
            match self.bridge.run_once_with_shutdown(self.running.clone()) {
                Ok(()) => {
                    tracing::info!("Pipeline completed normally (EOS), reconnecting immediately...");

                    // Update state
                    self.lifecycle.transition_to_connecting();
                    self.metrics.report_state_change(self.lifecycle.current_state());

                    // Reset backoff on successful run
                    current_backoff = self.backoff_policy.initial_delay();
                    reconnect_attempt = 0;
                }
                Err(e) => {
                    tracing::error!("Pipeline error: {}", e);

                    if !self.running.load(Ordering::SeqCst) {
                        break;
                    }

                    reconnect_attempt += 1;
                    self.metrics.report_reconnect_attempt();

                    // Update state to reconnecting
                    self.lifecycle.transition_to_reconnecting(
                        reconnect_attempt,
                        Some(e.to_string()),
                    );
                    self.metrics.report_state_change(self.lifecycle.current_state());

                    // Update metrics
                    self.metrics.report_backoff(current_backoff.as_secs_f64());
                    self.metrics.report_srt_state(false);

                    tracing::info!(
                        "Reconnecting in {:?} (attempt {})...",
                        current_backoff,
                        reconnect_attempt
                    );

                    std::thread::sleep(current_backoff);
                    current_backoff = self.backoff_policy.next_delay(current_backoff);
                }
            }

            // Update uptime if streaming
            if let Some(uptime) = self.lifecycle.uptime() {
                self.metrics.report_uptime(uptime.as_secs_f64());
            }
        }

        tracing::info!("Pipeline stopped");

        // Final state update
        self.lifecycle.transition_to_failed(Some("Stopped".to_string()));
        self.metrics.report_state_change(self.lifecycle.current_state());

        Ok(())
    }

    /// Stop the bridge
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.lifecycle.transition_to_failed(Some("Stopped by user".to_string()));
        self.metrics.report_state_change(self.lifecycle.current_state());
    }
}
