pub mod routes;

use axum::{routing::get, Router};

pub fn app() -> Router {
    Router::new()
        .route("/healthz", get(routes::healthz))
        .route("/v1/capabilities", get(routes::capabilities))
}
