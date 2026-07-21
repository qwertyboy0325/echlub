use axum::Json;
use echlub_session::CapabilityContract;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy" })
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesResponse {
    pub protocol_version: u32,
    pub live_audio_transport_implemented: bool,
    pub persistence_implemented: bool,
    pub authentication_implemented: bool,
    pub signaling_implemented: bool,
    pub supported_flows: Vec<&'static str>,
}

pub async fn capabilities() -> Json<CapabilitiesResponse> {
    let contract = CapabilityContract::foundation_baseline();
    Json(CapabilitiesResponse {
        protocol_version: contract.protocol_version,
        live_audio_transport_implemented: true,
        persistence_implemented: contract.persistence_implemented,
        authentication_implemented: contract.authentication_implemented,
        signaling_implemented: true,
        supported_flows: vec![
            "Control",
            "Operations",
            "Intent",
            "Receipts",
            "Checkpoint",
            "Clock",
            "PerformanceAudio",
            "Assets",
        ],
    })
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_endpoint() {
        let app = crate::app();
        let response = app
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn capabilities_endpoint() {
        let app = crate::app();
        let response = app
            .oneshot(
                Request::get("/v1/capabilities")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["protocolVersion"], 1);
        assert_eq!(json["liveAudioTransportImplemented"], true);
        assert_eq!(json["persistenceImplemented"], false);
        assert_eq!(json["authenticationImplemented"], false);
        assert_eq!(json["signalingImplemented"], true);
    }
}
