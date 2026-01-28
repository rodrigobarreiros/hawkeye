pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;

// Re-exports for convenience
pub use application::services::BridgeService;
pub use config::Config;
pub use domain::entities::{ConnectionLifecycle, StateTransition};
pub use domain::errors::{DomainError, Result};
pub use domain::ports::{MetricsReporter, StreamBridge};
pub use domain::value_objects::{BackoffPolicy, BridgeConfig, ConnectionState};
pub use infrastructure::gstreamer::{GStreamerBridge, PipelineBuilder};
pub use infrastructure::metrics::{serve_metrics, PrometheusReporter};
