use crate::domain::errors::{DomainError, Result};

/// Configuration for the RTSP to SRT bridge
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeConfig {
    rtsp_url: String,
    srt_url: String,
}

impl BridgeConfig {
    pub fn new(rtsp_url: String, srt_url: String) -> Result<Self> {
        Self::validate_rtsp_url(&rtsp_url)?;
        Self::validate_srt_url(&srt_url)?;

        Ok(Self { rtsp_url, srt_url })
    }

    pub fn rtsp_url(&self) -> &str {
        &self.rtsp_url
    }

    pub fn srt_url(&self) -> &str {
        &self.srt_url
    }

    fn validate_rtsp_url(url: &str) -> Result<()> {
        if !url.starts_with("rtsp://") {
            return Err(DomainError::InvalidRtspUrl(url.to_string()));
        }
        Ok(())
    }

    fn validate_srt_url(url: &str) -> Result<()> {
        if !url.starts_with("srt://") {
            return Err(DomainError::InvalidSrtUrl(url.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let result = BridgeConfig::new(
            "rtsp://localhost:8554/cam1".to_string(),
            "srt://localhost:9000".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_rejects_invalid_rtsp_url() {
        let result = BridgeConfig::new(
            "http://localhost:8554/cam1".to_string(),
            "srt://localhost:9000".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_invalid_srt_url() {
        let result = BridgeConfig::new(
            "rtsp://localhost:8554/cam1".to_string(),
            "tcp://localhost:9000".to_string(),
        );
        assert!(result.is_err());
    }
}
