use backend::executor::exec_code;
use backend::ipfs::fetch_from_ipfs;
use backend::models::{ExecuteResponse, ExecutionStatus};
use backend::otel::init_opentelemetry;

use axum::routing::{get, post};
use axum::{Json, Router, extract::Path};
use opentelemetry::global;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    init_opentelemetry().expect("Failed to initialize OpenTelemetry");

    let tracer = global::tracer("lgtm_tracer");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .with(telemetry)
        .init();

    let app = Router::new()
        .route("/api/v1/health", get(|| async { "OK" }))
        .route("/api/v1/code/{cid}", get(poll_status))
        .route("/api/v1/execute", post(exec_code))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to bind to address");

    info!(host = %host, port = %port, "Server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
}

/// Fetch status of a code execution from the queue.
async fn poll_status(Path(cid): Path<String>) -> Json<ExecuteResponse> {
    info!(cid = %cid, "Polling status for request");

    match fetch_from_ipfs(&cid).await {
        Ok(res) => axum::Json(ExecuteResponse {
            status: ExecutionStatus::Success,
            stdout: res.stdout,
            stderr: res.stderr,
            compilation_log: "".into(),
            execution_time_ms: res.execution_time_ms,
            ipfs_cid: Some(cid),
        }),
        Err(e) => {
            error!(error = %e, "Failed to retrieve CID from IPFS");
            axum::Json(ExecuteResponse {
                status: ExecutionStatus::RuntimeError,
                stdout: "".into(),
                stderr: format!("Failed to retrieve CID from IPFS: {}", e),
                compilation_log: "".into(),
                execution_time_ms: 42,
                ipfs_cid: Some(cid),
            })
        }
    }
}
