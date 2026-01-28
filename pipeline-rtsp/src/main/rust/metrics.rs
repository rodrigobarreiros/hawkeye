use prometheus::{
    IntCounter, IntGauge, Registry, TextEncoder, Encoder
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref ACTIVE_SESSIONS: IntGauge = IntGauge::new(
        "rtsp_active_sessions",
        "Number of active RTSP streaming sessions (server-side)"
    ).expect("metric can be created");

    pub static ref ACTIVE_CLIENTS: IntGauge = IntGauge::new(
        "rtsp_active_clients",
        "Number of currently connected RTSP clients"
    ).expect("metric can be created");

    pub static ref TOTAL_CONNECTIONS: IntCounter = IntCounter::new(
        "rtsp_client_connections_total",
        "Total number of RTSP client connections since server start"
    ).expect("metric can be created");

    pub static ref BYTES_SENT: IntCounter = IntCounter::new(
        "rtsp_bytes_sent_total",
        "Total bytes sent to RTSP clients"
    ).expect("metric can be created");
}

pub fn init_metrics() -> Result<(), prometheus::Error> {
    REGISTRY.register(Box::new(ACTIVE_SESSIONS.clone()))?;
    REGISTRY.register(Box::new(ACTIVE_CLIENTS.clone()))?;
    REGISTRY.register(Box::new(TOTAL_CONNECTIONS.clone()))?;
    REGISTRY.register(Box::new(BYTES_SENT.clone()))?;
    Ok(())
}

pub fn gather_metrics() -> Vec<u8> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {}", e);
        return b"# Error encoding metrics\n".to_vec();
    }
    buffer
}

pub async fn serve_metrics(port: u16) {
    use warp::Filter;

    // CORS configuration for browser access
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    let metrics_route = warp::path("metrics")
        .map(|| {
            let body = gather_metrics();
            warp::reply::with_header(
                body,
                "content-type",
                "text/plain; version=0.0.4; charset=utf-8"
            )
        });

    let health_route = warp::path("health")
        .map(|| warp::reply::with_status("OK".to_string(), warp::http::StatusCode::OK));

    let routes = metrics_route.or(health_route).with(cors);

    tracing::info!("Metrics server starting on port {}", port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

pub fn client_connected() {
    ACTIVE_CLIENTS.inc();
    TOTAL_CONNECTIONS.inc();
}

pub fn client_disconnected() {
    ACTIVE_CLIENTS.dec();
}
