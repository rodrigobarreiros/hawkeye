use crate::domain::value_objects::BridgeConfig;

pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Build the GStreamer pipeline string for RTSP to SRT conversion
    /// Uses H264 passthrough with MPEG-TS muxing for SRT transport
    pub fn build_pipeline_string(config: &BridgeConfig) -> String {
        // Use h264parse with config-interval=1 to insert SPS/PPS before every IDR frame
        // This ensures decoders can start at any keyframe
        // alignment=au ensures access unit alignment for proper MPEG-TS muxing
        // mpegtsmux alignment=7 aligns to 7 TS packets (1316 bytes) for SRT compatibility
        format!(
            "rtspsrc location={} latency=200 protocols=tcp ! \
             rtph264depay ! \
             h264parse config-interval=1 ! \
             video/x-h264,stream-format=byte-stream,alignment=au ! \
             mpegtsmux alignment=7 ! \
             srtsink uri=\"{}\" wait-for-connection=false",
            config.rtsp_url(),
            config.srt_url()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pipeline_string() {
        let config = BridgeConfig::new(
            "rtsp://localhost:8554/cam1".to_string(),
            "srt://localhost:9000".to_string(),
        )
        .unwrap();

        let pipeline = PipelineBuilder::build_pipeline_string(&config);

        assert!(pipeline.contains("rtspsrc location=rtsp://localhost:8554/cam1"));
        assert!(pipeline.contains("rtph264depay"));
        assert!(pipeline.contains("h264parse config-interval=1"));
        assert!(pipeline.contains("mpegtsmux alignment=7"));
        assert!(pipeline.contains("srtsink uri=\"srt://localhost:9000\""));
    }
}
