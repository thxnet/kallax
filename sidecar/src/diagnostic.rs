use std::{net::SocketAddr, time::Instant};

use axum::{extract::Extension, http::StatusCode, response::IntoResponse, routing, Json, Router};
use serde::Serialize;

use crate::peer_discoverer::{DiagnosticSnapshot, ErrorEntry, SharedDiagnostic, SharedErrorRing};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize)]
pub struct SidecarDiagnosticConfig {
    pub tracker_grpc_endpoint: String,
    pub rootchain_id: String,
    pub leafchain_id: Option<String>,
    pub allow_loopback_ip: bool,
    pub prefer_exposed_peers: bool,
    pub external_rootchain_p2p_endpoint: Option<String>,
    pub external_leafchain_p2p_endpoint: Option<String>,
    pub polling_interval_ms: u64,
    pub detected_public_ip: Option<String>,
}

#[derive(Clone)]
struct DiagnosticState {
    config: SidecarDiagnosticConfig,
    start_time: Instant,
    rootchain: SharedDiagnostic,
    leafchain: SharedDiagnostic,
    rootchain_errors: SharedErrorRing,
    leafchain_errors: SharedErrorRing,
}

#[derive(Serialize)]
struct DiagnosticResponse {
    version: &'static str,
    uptime_seconds: u64,
    config: SidecarDiagnosticConfig,
    rootchain: Option<DiagnosticSnapshot>,
    leafchain: Option<DiagnosticSnapshot>,
    recent_errors: Vec<ErrorEntry>,
}

#[derive(Serialize)]
struct HealthResponse {
    healthy: bool,
}

async fn get_diagnostic(Extension(state): Extension<DiagnosticState>) -> impl IntoResponse {
    let rootchain = state.rootchain.lock().await.clone();
    let leafchain = state.leafchain.lock().await.clone();

    let mut recent_errors = state.rootchain_errors.lock().await.entries();
    recent_errors.extend(state.leafchain_errors.lock().await.entries());
    recent_errors.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    (
        StatusCode::OK,
        Json(DiagnosticResponse {
            version: VERSION,
            uptime_seconds: state.start_time.elapsed().as_secs(),
            config: state.config.clone(),
            rootchain,
            leafchain,
            recent_errors,
        }),
    )
}

async fn get_health(Extension(state): Extension<DiagnosticState>) -> impl IntoResponse {
    let rootchain = state.rootchain.lock().await;
    let leafchain = state.leafchain.lock().await;

    // Healthy if rootchain has had at least one successful poll
    let healthy = rootchain.as_ref().is_some_and(|s| s.health.last_successful_poll.is_some());

    // Also check leafchain if configured
    let healthy = if state.config.leafchain_id.is_some() {
        healthy && leafchain.as_ref().is_some_and(|s| s.health.last_successful_poll.is_some())
    } else {
        healthy
    };

    let status = if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (status, Json(HealthResponse { healthy }))
}

#[allow(clippy::too_many_arguments)]
pub async fn serve(
    listen_address: SocketAddr,
    config: SidecarDiagnosticConfig,
    start_time: Instant,
    rootchain: SharedDiagnostic,
    leafchain: SharedDiagnostic,
    rootchain_errors: SharedErrorRing,
    leafchain_errors: SharedErrorRing,
    shutdown: sigfinn::Shutdown,
) -> sigfinn::ExitStatus<crate::Error> {
    let state = DiagnosticState {
        config,
        start_time,
        rootchain,
        leafchain,
        rootchain_errors,
        leafchain_errors,
    };

    let router = Router::new()
        .route("/diagnostic", routing::get(get_diagnostic))
        .route("/health", routing::get(get_health))
        .layer(axum::Extension(state))
        .into_make_service();

    tracing::info!("Listen Diagnostic API on {listen_address}");

    match axum::Server::try_bind(&listen_address) {
        Ok(server) => match server.serve(router).with_graceful_shutdown(shutdown).await {
            Ok(()) => {
                tracing::info!("Diagnostic server shut down gracefully");
                sigfinn::ExitStatus::Success
            }
            Err(err) => {
                tracing::error!("Diagnostic server error: {err}");
                sigfinn::ExitStatus::Success
            }
        },
        Err(err) => {
            tracing::error!("Failed to bind diagnostic server to {listen_address}: {err}");
            sigfinn::ExitStatus::Success
        }
    }
}
