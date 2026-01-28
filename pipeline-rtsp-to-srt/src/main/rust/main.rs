use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::oneshot;
use tracing::{error, info};
use warp::Filter;

use pipeline_rtsp_to_srt::{BridgeService, Config, GStreamerBridge, PrometheusReporter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();
    config.validate()?;

    // Initialize logging
    let filter = if config.verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .init();

    // Initialize GStreamer (infrastructure concern)
    gstreamer::init()?;

    // Initialize metrics
    PrometheusReporter::init_metrics()?;

    info!("Starting RTSP to SRT pipeline");
    info!("  RTSP source: {}", config.rtsp_url);
    info!("  SRT destination: {}", config.srt_url);
    info!("  Metrics port: {}", config.metrics_port);

    // Convert CLI config to domain configs
    let bridge_config = config
        .to_bridge_config()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let backoff_policy = config
        .to_backoff_policy()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create infrastructure implementations (dependency injection)
    let bridge = Box::new(GStreamerBridge::new(bridge_config));
    let metrics_reporter = Arc::new(PrometheusReporter::new());

    // Create application service
    let mut bridge_service = BridgeService::new(bridge, backoff_policy, metrics_reporter);
    let running = bridge_service.running_flag();

    // Set up graceful shutdown
    let running_for_signal = running.clone();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_tx = Arc::new(tokio::sync::Mutex::new(Some(shutdown_tx)));

    // Handle Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        info!("Received shutdown signal");
        running_for_signal.store(false, Ordering::SeqCst);
        if let Some(tx) = shutdown_tx_clone.lock().await.take() {
            let _ = tx.send(());
        }
    });

    // Start metrics server
    let metrics_port = config.metrics_port;
    let metrics_server = {
        // CORS configuration for browser access
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);

        let health_route = warp::path("health")
            .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

        let metrics_route = warp::path("metrics").map(|| {
            use prometheus::Encoder;
            let encoder = prometheus::TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            warp::reply::with_header(
                String::from_utf8(buffer).unwrap(),
                "Content-Type",
                "text/plain; charset=utf-8",
            )
        });

        let routes = health_route.or(metrics_route).with(cors);

        let (addr, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], metrics_port), async {
                shutdown_rx.await.ok();
            });

        info!("Metrics server listening on http://{}", addr);
        tokio::spawn(server)
    };

    // Run bridge in a blocking thread (GStreamer uses synchronous APIs)
    let pipeline_handle = tokio::task::spawn_blocking(move || {
        if let Err(e) = bridge_service.run_with_reconnect() {
            error!("Pipeline error: {}", e);
        }
    });

    // Wait for pipeline to complete
    pipeline_handle.await?;

    // Signal shutdown to metrics server
    if let Some(tx) = shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }

    // Wait for metrics server to shut down
    metrics_server.await?;

    info!("Pipeline shutdown complete");
    Ok(())
}
