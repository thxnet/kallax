mod v1;

use axum::{routing, Router};
pub use v1::InsertLeafchainPeerAddressRequest;

pub fn v1() -> Router {
    Router::new().nest(
        "/v1/leafchain",
        Router::new()
            .route("/:chain_id/chain-spec", routing::get(self::v1::get_chain_spec))
            .route("/:chain_id/peers", routing::get(self::v1::get_peers))
            .route("/:chain_id/insert", routing::post(self::v1::insert_peers)),
    )
}
