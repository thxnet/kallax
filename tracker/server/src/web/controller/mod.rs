mod diagnostic;
pub mod leafchain;
pub mod rootchain;

use axum::{routing, Router};

pub fn api_v1_router() -> Router {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(self::rootchain::v1())
            .merge(self::leafchain::v1())
            .route("/v1/diagnostic", routing::get(self::diagnostic::get_diagnostic)),
    )
}
