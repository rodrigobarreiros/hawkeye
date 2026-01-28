use crate::domain::value_objects::{ContainerFormat, StreamConfig, VideoCodec};

pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Convert domain config to GStreamer pipeline string
    pub fn build_launch_string(config: &StreamConfig) -> String {
        let demuxer = Self::demuxer_for_container(config.container());
        let parser = Self::parser_for_codec(config.codec());
        let payloader = Self::payloader_for_codec(config.codec());

        format!(
            "( filesrc location={} ! {} ! {} ! {} name=pay0 pt={} )",
            config.source_path().display(),
            demuxer,
            parser,
            payloader,
            config.rtp_payload_type()
        )
    }

    fn demuxer_for_container(container: &ContainerFormat) -> &'static str {
        match container {
            ContainerFormat::MP4 => "qtdemux",
            ContainerFormat::MKV => "matroskademux",
        }
    }

    fn parser_for_codec(codec: &VideoCodec) -> &'static str {
        match codec {
            VideoCodec::H264 => "h264parse config-interval=-1",
            VideoCodec::H265 => "h265parse config-interval=-1",
        }
    }

    fn payloader_for_codec(codec: &VideoCodec) -> &'static str {
        match codec {
            VideoCodec::H264 => "rtph264pay",
            VideoCodec::H265 => "rtph265pay",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_h264_mp4_pipeline() {
        let config = StreamConfig::new(PathBuf::from("/test/video.mp4"));
        let pipeline = PipelineBuilder::build_launch_string(&config);

        assert!(pipeline.contains("filesrc location=/test/video.mp4"));
        assert!(pipeline.contains("qtdemux"));
        assert!(pipeline.contains("h264parse config-interval=-1"));
        assert!(pipeline.contains("rtph264pay"));
        assert!(pipeline.contains("pt=96"));
    }

    #[test]
    fn test_build_h265_pipeline() {
        let config =
            StreamConfig::new(PathBuf::from("/test/video.mp4")).with_codec(VideoCodec::H265);
        let pipeline = PipelineBuilder::build_launch_string(&config);

        assert!(pipeline.contains("h265parse config-interval=-1"));
        assert!(pipeline.contains("rtph265pay"));
    }

    #[test]
    fn test_build_mkv_pipeline() {
        let config =
            StreamConfig::new(PathBuf::from("/test/video.mkv")).with_container(ContainerFormat::MKV);
        let pipeline = PipelineBuilder::build_launch_string(&config);

        assert!(pipeline.contains("matroskademux"));
    }

    #[test]
    fn test_custom_payload_type() {
        let config =
            StreamConfig::new(PathBuf::from("/test/video.mp4")).with_rtp_payload_type(97);
        let pipeline = PipelineBuilder::build_launch_string(&config);

        assert!(pipeline.contains("pt=97"));
    }
}
