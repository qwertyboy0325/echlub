pub mod routes;
pub mod signaling;

use axum::{routing::get, Router};
use signaling::SignalingState;

pub fn app() -> Router {
    let signaling_state = SignalingState::default();
    Router::new()
        .route("/healthz", get(routes::healthz))
        .route("/v1/capabilities", get(routes::capabilities))
        .route("/v1/signaling/ws", get(signaling::signaling_ws_handler))
        .with_state(signaling_state)
}
