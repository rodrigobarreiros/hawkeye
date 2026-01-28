use warp::Filter;

use super::PrometheusReporter;

/// Health check response structure
#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

pub async fn serve_metrics(port: u16) {
    // CORS configuration for browser access
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    let metrics_route = warp::path("metrics").map(|| {
        let body = PrometheusReporter::gather_metrics();
        warp::reply::with_header(body, "content-type", "text/plain; version=0.0.4; charset=utf-8")
    });

    let health_route = warp::path("health").map(|| {
        let response = HealthResponse {
            status: "healthy",
            service: "pipeline-rtsp",
            version: env!("CARGO_PKG_VERSION"),
        };
        warp::reply::json(&response)
    });

    // Liveness probe endpoint (minimal check - is the process running?)
    let liveness_route =
        warp::path("livez").map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

    // Readiness probe endpoint (can the service accept traffic?)
    let readiness_route = warp::path("readyz").map(|| {
        let response = HealthResponse {
            status: "ready",
            service: "pipeline-rtsp",
            version: env!("CARGO_PKG_VERSION"),
        };
        warp::reply::json(&response)
    });

    let routes = metrics_route
        .or(health_route)
        .or(liveness_route)
        .or(readiness_route)
        .with(cors);

    tracing::info!("Metrics server starting on port {}", port);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
