use echlub_control_plane::{app, config::ControlPlaneConfig};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let config = ControlPlaneConfig::from_env().expect("invalid control plane config");
    tracing::info!("binding control plane to {}", config.bind_addr);
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .expect("bind");
    axum::serve(listener, app()).await.expect("serve");
}
