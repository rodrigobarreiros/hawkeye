use std::time::Duration;

use clap::Parser;

use crate::domain::value_objects::{BackoffPolicy, BridgeConfig};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pipeline-rtsp-to-srt",
    version = "0.1.0",
    author = "Hawkeye Video Pipeline",
    about = "RTSP to SRT bridge with automatic reconnection (Pipeline 2)"
)]
pub struct Config {
    /// RTSP source URL
    #[arg(
        long,
        env = "RTSP_URL",
        default_value = "rtsp://pipeline-rtsp:8554/cam1"
    )]
    pub rtsp_url: String,

    /// SRT destination URL
    #[arg(
        long,
        env = "SRT_URL",
        default_value = "srt://mediamtx:9000?mode=caller&streamid=publish:cam1&latency=200"
    )]
    pub srt_url: String,

    /// Metrics server port
    #[arg(long, env = "METRICS_PORT", default_value = "9002")]
    pub metrics_port: u16,

    /// Initial reconnection delay in seconds
    #[arg(long, default_value = "1")]
    pub reconnect_initial_delay: u64,

    /// Maximum reconnection delay in seconds
    #[arg(long, default_value = "30")]
    pub reconnect_max_delay: u64,

    /// Reconnection backoff multiplier
    #[arg(long, default_value = "2.0")]
    pub reconnect_multiplier: f64,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Minimum allowed port (ports below 1024 are privileged)
const MIN_USER_PORT: u16 = 1024;

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.rtsp_url.starts_with("rtsp://") {
            anyhow::bail!("RTSP URL must start with rtsp://");
        }

        if !self.srt_url.starts_with("srt://") {
            anyhow::bail!("SRT URL must start with srt://");
        }

        Self::validate_port(self.metrics_port, "metrics")?;

        if self.reconnect_multiplier <= 1.0 {
            anyhow::bail!("Reconnect multiplier must be > 1.0");
        }

        if self.reconnect_initial_delay == 0 {
            anyhow::bail!("Initial reconnection delay cannot be 0");
        }

        if self.reconnect_max_delay < self.reconnect_initial_delay {
            anyhow::bail!(
                "Maximum reconnection delay ({}) cannot be less than initial delay ({})",
                self.reconnect_max_delay,
                self.reconnect_initial_delay
            );
        }

        Ok(())
    }

    fn validate_port(port: u16, name: &str) -> anyhow::Result<()> {
        if port == 0 {
            anyhow::bail!("Invalid {} port: port cannot be 0", name);
        }
        if port < MIN_USER_PORT {
            anyhow::bail!(
                "Invalid {} port: {} is a privileged port (< {}). Use a port >= {}",
                name,
                port,
                MIN_USER_PORT,
                MIN_USER_PORT
            );
        }
        Ok(())
    }

    pub fn to_bridge_config(&self) -> crate::domain::errors::Result<BridgeConfig> {
        BridgeConfig::new(self.rtsp_url.clone(), self.srt_url.clone())
    }

    pub fn to_backoff_policy(&self) -> crate::domain::errors::Result<BackoffPolicy> {
        BackoffPolicy::new(
            Duration::from_secs(self.reconnect_initial_delay),
            Duration::from_secs(self.reconnect_max_delay),
            self.reconnect_multiplier,
        )
    }
}
