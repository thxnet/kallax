mod leafchain;
mod rootchain;

use axum::Router;

pub fn api_v1_router() -> Router {
    Router::new()
        .nest("/api", Router::new().merge(self::rootchain::v1()).merge(self::leafchain::v1()))
}
