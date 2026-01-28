use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

use pipeline_rtsp::{
    Config, GStreamerRtspServer, PrometheusReporter, ServerConfig, StreamConfig, StreamingService,
    serve_metrics,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();

    // Initialize logging
    if config.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    info!("Starting Pipeline-RTSP v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration: {:?}", config);

    // Validate CLI configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(e);
    }

    info!("Configuration validated");

    // Initialize GStreamer (infrastructure concern)
    gstreamer::init()?;
    info!("GStreamer initialized");

    // Initialize metrics
    PrometheusReporter::init_metrics()?;
    info!("Metrics initialized");

    // Start metrics server
    let metrics_port = config.metrics_port;
    tokio::spawn(async move {
        serve_metrics(metrics_port).await;
    });
    info!("Metrics server started on port {}", config.metrics_port);

    // Create infrastructure implementations (dependency injection)
    let server = Box::new(GStreamerRtspServer::new());
    let metrics_reporter = Arc::new(PrometheusReporter::new());

    // Create application service
    let streaming_service = StreamingService::new(server, metrics_reporter);

    // Convert CLI config to domain configs
    let stream_config = StreamConfig::new(config.video_path.clone());
    let server_config = ServerConfig::new(config.rtsp_port, config.mount_point.clone())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Start streaming (use case)
    let session = streaming_service
        .start_streaming(stream_config, server_config)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    info!("-------------------------------------------------------");
    info!("RTSP Server Ready");
    info!(
        "   URL:     rtsp://0.0.0.0:{}{}",
        config.rtsp_port, config.mount_point
    );
    info!("   Video:   {:?}", config.video_path);
    info!("   Session: {}", session.id());
    info!("   Metrics: http://0.0.0.0:{}/metrics", config.metrics_port);
    info!("   Health:  http://0.0.0.0:{}/health", config.metrics_port);
    info!("-------------------------------------------------------");

    // Create main loop for GStreamer
    let main_loop = glib::MainLoop::new(None, false);
    let main_loop_clone = main_loop.clone();

    // Handle graceful shutdown
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received (Ctrl+C)");
                main_loop_clone.quit();
            }
            Err(err) => {
                error!("Failed to listen for shutdown signal: {}", err);
            }
        }
    });

    // Run main loop
    main_loop.run();

    // Graceful shutdown
    streaming_service.stop_streaming().await.ok();

    info!("Server stopped gracefully");
    Ok(())
}
