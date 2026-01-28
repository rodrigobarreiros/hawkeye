use std::path::PathBuf;

use super::{ContainerFormat, VideoCodec};
use crate::domain::errors::{DomainError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct StreamConfig {
    source_path: PathBuf,
    codec: VideoCodec,
    container: ContainerFormat,
    rtp_payload_type: u8,
}

impl StreamConfig {
    pub fn new(source_path: PathBuf) -> Self {
        Self {
            source_path,
            codec: VideoCodec::default(),
            container: ContainerFormat::default(),
            rtp_payload_type: 96,
        }
    }

    pub fn with_codec(mut self, codec: VideoCodec) -> Self {
        self.codec = codec;
        self
    }

    pub fn with_container(mut self, container: ContainerFormat) -> Self {
        self.container = container;
        self
    }

    pub fn with_rtp_payload_type(mut self, pt: u8) -> Self {
        self.rtp_payload_type = pt;
        self
    }

    pub fn source_path(&self) -> &PathBuf {
        &self.source_path
    }

    pub fn codec(&self) -> &VideoCodec {
        &self.codec
    }

    pub fn container(&self) -> &ContainerFormat {
        &self.container
    }

    pub fn rtp_payload_type(&self) -> u8 {
        self.rtp_payload_type
    }

    /// Pure validation logic (domain concern)
    pub fn validate(&self) -> Result<()> {
        if !self.source_path.exists() {
            return Err(DomainError::InvalidPath(self.source_path.clone()));
        }

        if !self.source_path.is_file() {
            return Err(DomainError::PathNotFile(self.source_path.clone()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = StreamConfig::new(PathBuf::from("/test/video.mp4"));

        assert_eq!(*config.codec(), VideoCodec::H264);
        assert_eq!(*config.container(), ContainerFormat::MP4);
        assert_eq!(config.rtp_payload_type(), 96);
    }

    #[test]
    fn test_with_codec() {
        let config =
            StreamConfig::new(PathBuf::from("/test/video.mp4")).with_codec(VideoCodec::H265);

        assert_eq!(*config.codec(), VideoCodec::H265);
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let config = StreamConfig::new(PathBuf::from("/nonexistent/video.mp4"));
        let result = config.validate();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidPath(_)));
    }

    #[test]
    fn test_validate_existing_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        let config = StreamConfig::new(temp_file.path().to_path_buf());
        let result = config.validate();

        assert!(result.is_ok());
    }
}
