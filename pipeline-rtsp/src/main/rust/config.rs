use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pipeline-rtsp",
    version = "0.1.0",
    author = "Hawkeye Video Pipeline",
    about = "RTSP server for video streaming (Pipeline 1)"
)]
pub struct Config {
    /// Path to input video file (MP4 with H.264/AAC)
    #[arg(
        short = 'i',
        long = "video-path",
        env = "VIDEO_PATH",
        default_value = "/app/resources/camera1.mp4"
    )]
    pub video_path: PathBuf,

    /// RTSP server port
    #[arg(long, env = "RTSP_PORT", default_value = "8554")]
    pub rtsp_port: u16,

    /// RTSP mount point
    #[arg(long, env = "RTSP_MOUNT_POINT", default_value = "/cam1")]
    pub mount_point: String,

    /// Metrics server port
    #[arg(long, env = "METRICS_PORT", default_value = "9001")]
    pub metrics_port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Minimum allowed port (ports below 1024 are privileged)
const MIN_USER_PORT: u16 = 1024;

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.video_path.exists() {
            anyhow::bail!("Video file not found: {:?}", self.video_path);
        }

        if !self.video_path.is_file() {
            anyhow::bail!("Video path is not a file: {:?}", self.video_path);
        }

        Self::validate_port(self.rtsp_port, "RTSP")?;
        Self::validate_port(self.metrics_port, "metrics")?;

        if self.rtsp_port == self.metrics_port {
            anyhow::bail!("RTSP port and metrics port cannot be the same");
        }

        Self::validate_mount_point(&self.mount_point)?;

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

    fn validate_mount_point(mount_point: &str) -> anyhow::Result<()> {
        if !mount_point.starts_with('/') {
            anyhow::bail!("Mount point must start with '/': {}", mount_point);
        }

        // Validate mount point contains only allowed characters (alphanumeric, /, -, _)
        let valid_chars = mount_point
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_');

        if !valid_chars {
            anyhow::bail!(
                "Mount point contains invalid characters: {}. Only alphanumeric, '/', '-', and '_' are allowed",
                mount_point
            );
        }

        // Check for double slashes or trailing slash (except root)
        if mount_point.contains("//") {
            anyhow::bail!("Mount point cannot contain double slashes: {}", mount_point);
        }

        if mount_point.len() > 1 && mount_point.ends_with('/') {
            anyhow::bail!("Mount point cannot end with '/': {}", mount_point);
        }

        Ok(())
    }
}
