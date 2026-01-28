use pipeline_rtsp_to_srt::{
    BackoffPolicy, BridgeConfig, ConnectionLifecycle, ConnectionState, PipelineBuilder,
};
use std::time::Duration;

#[test]
fn test_gstreamer_init() {
    assert!(gstreamer::init().is_ok());
}

#[test]
fn test_bridge_config_validation() {
    // Valid config
    let result = BridgeConfig::new(
        "rtsp://localhost:8554/cam1".to_string(),
        "srt://localhost:9000".to_string(),
    );
    assert!(result.is_ok());

    // Invalid RTSP URL
    let result = BridgeConfig::new(
        "http://localhost:8554/cam1".to_string(),
        "srt://localhost:9000".to_string(),
    );
    assert!(result.is_err());

    // Invalid SRT URL
    let result = BridgeConfig::new(
        "rtsp://localhost:8554/cam1".to_string(),
        "tcp://localhost:9000".to_string(),
    );
    assert!(result.is_err());
}

#[test]
fn test_build_pipeline_string_contains_elements() {
    let config = BridgeConfig::new(
        "rtsp://localhost:8554/cam1".to_string(),
        "srt://localhost:9000?mode=caller".to_string(),
    )
    .unwrap();

    let pipeline = PipelineBuilder::build_pipeline_string(&config);

    assert!(pipeline.contains("rtspsrc"));
    assert!(pipeline.contains("location=rtsp://localhost:8554/cam1"));
    assert!(pipeline.contains("rtph264depay"));
    assert!(pipeline.contains("h264parse"));
    assert!(pipeline.contains("mpegtsmux"));
    assert!(pipeline.contains("srtsink"));
    assert!(pipeline.contains("srt://localhost:9000"));
}

#[test]
fn test_backoff_policy_default() {
    let policy = BackoffPolicy::default();
    assert_eq!(policy.initial_delay(), Duration::from_secs(1));
    assert_eq!(policy.max_delay(), Duration::from_secs(30));
    assert_eq!(policy.multiplier(), 2.0);
}

#[test]
fn test_backoff_exponential_growth() {
    let policy = BackoffPolicy::default();

    let d1 = Duration::from_secs(1);
    let d2 = policy.next_delay(d1);
    assert_eq!(d2, Duration::from_secs(2));

    let d3 = policy.next_delay(d2);
    assert_eq!(d3, Duration::from_secs(4));

    let d4 = policy.next_delay(d3);
    assert_eq!(d4, Duration::from_secs(8));

    let d5 = policy.next_delay(d4);
    assert_eq!(d5, Duration::from_secs(16));
}

#[test]
fn test_backoff_max_cap() {
    let policy = BackoffPolicy::new(
        Duration::from_secs(1),
        Duration::from_secs(10),
        2.0,
    )
    .unwrap();

    let large = Duration::from_secs(8);
    let capped = policy.next_delay(large);
    assert_eq!(capped, Duration::from_secs(10)); // Capped at max

    let very_large = Duration::from_secs(100);
    let still_capped = policy.next_delay(very_large);
    assert_eq!(still_capped, Duration::from_secs(10));
}

#[test]
fn test_backoff_rejects_invalid_multiplier() {
    let result = BackoffPolicy::new(Duration::from_secs(1), Duration::from_secs(30), 1.0);
    assert!(result.is_err());

    let result = BackoffPolicy::new(Duration::from_secs(1), Duration::from_secs(30), 0.5);
    assert!(result.is_err());
}

#[test]
fn test_connection_lifecycle_initial_state() {
    let lifecycle = ConnectionLifecycle::new();
    assert_eq!(*lifecycle.current_state(), ConnectionState::Idle);
    assert_eq!(lifecycle.transition_count(), 0);
}

#[test]
fn test_connection_lifecycle_transitions() {
    let mut lifecycle = ConnectionLifecycle::new();

    lifecycle.transition_to_connecting();
    assert_eq!(*lifecycle.current_state(), ConnectionState::Connecting);
    assert_eq!(lifecycle.transition_count(), 1);

    lifecycle.transition_to_streaming();
    assert_eq!(*lifecycle.current_state(), ConnectionState::Streaming);
    assert_eq!(lifecycle.transition_count(), 2);

    lifecycle.transition_to_reconnecting(1, Some("test error".to_string()));
    assert!(matches!(
        *lifecycle.current_state(),
        ConnectionState::Reconnecting { attempt: 1 }
    ));
    assert_eq!(lifecycle.transition_count(), 3);
}

#[test]
fn test_connection_state_methods() {
    assert!(!ConnectionState::Idle.is_streaming());
    assert!(ConnectionState::Streaming.is_streaming());
    assert!(!ConnectionState::Reconnecting { attempt: 1 }.is_streaming());

    assert!(!ConnectionState::Idle.is_problematic());
    assert!(!ConnectionState::Streaming.is_problematic());
    assert!(ConnectionState::Reconnecting { attempt: 1 }.is_problematic());
    assert!(ConnectionState::Failed.is_problematic());
}

#[test]
fn test_gstreamer_pipeline_parsing() {
    gstreamer::init().unwrap();

    let config = BridgeConfig::new(
        "rtsp://localhost:8554/cam1".to_string(),
        "srt://localhost:9000?mode=caller&latency=200".to_string(),
    )
    .unwrap();

    let pipeline_str = PipelineBuilder::build_pipeline_string(&config);

    // Verify the pipeline string can be parsed by GStreamer
    let result = gstreamer::parse::launch(&pipeline_str);
    assert!(
        result.is_ok(),
        "Pipeline should parse successfully: {:?}",
        result.err()
    );
}
