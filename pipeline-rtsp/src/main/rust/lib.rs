pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;

// Re-exports for convenience
pub use application::services::StreamingService;
pub use config::Config;
pub use domain::entities::{SessionState, StreamSession};
pub use domain::errors::{DomainError, Result};
pub use domain::ports::{MetricsReporter, StreamingServer};
pub use domain::value_objects::{ContainerFormat, ServerConfig, StreamConfig, VideoCodec};
pub use infrastructure::gstreamer::{GStreamerRtspServer, PipelineBuilder};
pub use infrastructure::metrics::{serve_metrics, PrometheusReporter};
