use std::sync::LazyLock;

use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};

use crate::domain::entities::StreamSession;
use crate::domain::ports::MetricsReporter;

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);
pub static ACTIVE_SESSIONS: LazyLock<IntGauge> = LazyLock::new(|| {
    IntGauge::new(
        "rtsp_active_sessions",
        "Number of active RTSP streaming sessions (server-side)",
    )
    .expect("metric can be created")
});
pub static ACTIVE_CLIENTS: LazyLock<IntGauge> = LazyLock::new(|| {
    IntGauge::new(
        "rtsp_active_clients",
        "Number of currently connected RTSP clients",
    )
    .expect("metric can be created")
});
pub static TOTAL_CONNECTIONS: LazyLock<IntCounter> = LazyLock::new(|| {
    IntCounter::new(
        "rtsp_client_connections_total",
        "Total number of RTSP client connections since server start",
    )
    .expect("metric can be created")
});
pub static BYTES_SENT: LazyLock<IntCounter> = LazyLock::new(|| {
    IntCounter::new(
        "rtsp_bytes_sent_total",
        "Total bytes sent to RTSP clients",
    )
    .expect("metric can be created")
});

pub struct PrometheusReporter;

impl PrometheusReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn init_metrics() -> Result<(), prometheus::Error> {
        REGISTRY.register(Box::new(ACTIVE_SESSIONS.clone()))?;
        REGISTRY.register(Box::new(ACTIVE_CLIENTS.clone()))?;
        REGISTRY.register(Box::new(TOTAL_CONNECTIONS.clone()))?;
        REGISTRY.register(Box::new(BYTES_SENT.clone()))?;
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
    fn report_session_started(&self, _session: &StreamSession) {
        ACTIVE_SESSIONS.inc();
    }

    fn report_session_stopped(&self, _session: &StreamSession) {
        ACTIVE_SESSIONS.dec();
    }

    fn report_client_connected(&self) {
        ACTIVE_CLIENTS.inc();
        TOTAL_CONNECTIONS.inc();
    }

    fn report_client_disconnected(&self) {
        ACTIVE_CLIENTS.dec();
    }
}
