use lazy_static::lazy_static;
use prometheus::{Encoder, Gauge, IntCounter, IntGauge, Registry, TextEncoder};

use crate::domain::ports::MetricsReporter;
use crate::domain::value_objects::ConnectionState;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Connection state (0=Idle, 1=Connecting, 2=Streaming, 3=Reconnecting, 4=Failed)
    pub static ref CONNECTION_STATE: Gauge = Gauge::new(
        "rtsp_srt_connection_state",
        "Current connection state"
    ).expect("metric can be created");

    // Total reconnection attempts
    pub static ref RECONNECT_ATTEMPTS: IntCounter = IntCounter::new(
        "reconnect_attempts_total",
        "Total number of reconnection attempts"
    ).expect("metric can be created");

    // Current backoff delay in seconds
    pub static ref BACKOFF_SECONDS: Gauge = Gauge::new(
        "reconnect_backoff_seconds",
        "Current reconnection backoff delay"
    ).expect("metric can be created");

    // Pipeline uptime
    pub static ref UPTIME_SECONDS: Gauge = Gauge::new(
        "pipeline_uptime_seconds",
        "Time since pipeline started streaming"
    ).expect("metric can be created");

    // SRT publish state (0=disconnected, 1=connected)
    pub static ref SRT_PUBLISH_STATE: IntGauge = IntGauge::new(
        "srt_publish_state",
        "SRT publish connection state"
    ).expect("metric can be created");
}

pub struct PrometheusReporter;

impl PrometheusReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn init_metrics() -> Result<(), prometheus::Error> {
        REGISTRY.register(Box::new(CONNECTION_STATE.clone()))?;
        REGISTRY.register(Box::new(RECONNECT_ATTEMPTS.clone()))?;
        REGISTRY.register(Box::new(BACKOFF_SECONDS.clone()))?;
        REGISTRY.register(Box::new(UPTIME_SECONDS.clone()))?;
        REGISTRY.register(Box::new(SRT_PUBLISH_STATE.clone()))?;
        Ok(())
    }

    pub fn gather_metrics() -> Vec<u8> {
        let encoder = TextEncoder::new();
        let metric_families = REGISTRY.gather();
        let mut buffer = vec![];
        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            tracing::error!("Failed to encode metrics: {}", e);
            return b"# Error encoding metrics\n".to_vec();
        }
        buffer
    }
}

impl Default for PrometheusReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsReporter for PrometheusReporter {
    fn report_state_change(&self, state: &ConnectionState) {
        CONNECTION_STATE.set(state.as_metric());
    }

    fn report_reconnect_attempt(&self) {
        RECONNECT_ATTEMPTS.inc();
    }

    fn report_backoff(&self, delay_secs: f64) {
        BACKOFF_SECONDS.set(delay_secs);
    }

    fn report_srt_state(&self, connected: bool) {
        SRT_PUBLISH_STATE.set(if connected { 1 } else { 0 });
    }

    fn report_uptime(&self, uptime_secs: f64) {
        UPTIME_SECONDS.set(uptime_secs);
    }
}
