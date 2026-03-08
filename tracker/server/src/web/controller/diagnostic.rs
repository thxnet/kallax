use std::collections::HashMap;

use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::web::extension::{
    LeafchainPeerAddressBook, RootchainPeerAddressBook, TrackerConfig, TrackerStartTime,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
struct DiagnosticResponse {
    version: &'static str,
    uptime_seconds: u64,
    config: TrackerConfig,
    rootchain: ChainSummary,
    leafchain: ChainSummary,
}

#[derive(Serialize)]
struct ChainSummary {
    chain_count: usize,
    total_peer_count: usize,
    peer_count_per_chain: HashMap<String, usize>,
}

pub async fn get_diagnostic(
    Extension(config): Extension<TrackerConfig>,
    Extension(start_time): Extension<TrackerStartTime>,
    Extension(RootchainPeerAddressBook(rootchain_book)): Extension<RootchainPeerAddressBook>,
    Extension(LeafchainPeerAddressBook(leafchain_book)): Extension<LeafchainPeerAddressBook>,
) -> impl IntoResponse {
    let rootchain_counts = rootchain_book.peer_counts().await;
    let leafchain_counts = leafchain_book.peer_counts().await;

    let rootchain = ChainSummary {
        chain_count: rootchain_counts.len(),
        total_peer_count: rootchain_counts.values().sum(),
        peer_count_per_chain: rootchain_counts,
    };

    let leafchain = ChainSummary {
        chain_count: leafchain_counts.len(),
        total_peer_count: leafchain_counts.values().sum(),
        peer_count_per_chain: leafchain_counts,
    };

    (
        StatusCode::OK,
        Json(DiagnosticResponse {
            version: VERSION,
            uptime_seconds: start_time.0.elapsed().as_secs(),
            config,
            rootchain,
            leafchain,
        }),
    )
}
