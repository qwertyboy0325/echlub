use echlub_control_plane::signaling::{validate_ids, validate_origin_header};

#[test]
fn validate_ids_rejects_empty_session() {
    assert!(validate_ids("", "peer1").is_err());
}

#[test]
fn validate_ids_rejects_empty_peer() {
    assert!(validate_ids("session1", "").is_err());
}

#[test]
fn validate_ids_accepts_valid() {
    assert!(validate_ids("session-1", "peer_a").is_ok());
}

#[test]
fn validate_ids_rejects_invalid_chars() {
    assert!(validate_ids("session!", "peer1").is_err());
}

#[test]
fn validate_origin_allows_localhost() {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "origin",
        axum::http::HeaderValue::from_static("http://localhost:5173"),
    );
    assert!(validate_origin_header(&headers, &[]));
}

#[test]
fn validate_origin_allows_missing() {
    let headers = axum::http::HeaderMap::new();
    assert!(validate_origin_header(&headers, &[]));
}

#[test]
fn validate_origin_rejects_external() {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "origin",
        axum::http::HeaderValue::from_static("https://evil.example.com"),
    );
    assert!(!validate_origin_header(&headers, &[]));
}

#[test]
fn validate_origin_rejects_deceptive_localhost() {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "origin",
        axum::http::HeaderValue::from_static("http://localhost.attacker.example:5173"),
    );
    assert!(!validate_origin_header(&headers, &[]));
}

#[test]
fn validate_origin_accepts_configured_lan_origin() {
    let extras = vec!["http://192.168.1.10:5173".to_string()];
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "origin",
        axum::http::HeaderValue::from_static("http://192.168.1.10:5173"),
    );
    assert!(validate_origin_header(&headers, &extras));
}

#[test]
fn validate_ids_maps_to_bad_request() {
    let err = validate_ids("", "peer").unwrap_err();
    assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
}
