use serde_json::Value;

const LIVE_FORBIDDEN_KEYS: &[&str] = &[
    "sdp",
    "ice",
    "candidate",
    "deviceid",
    "device_id",
    "groupid",
    "group_id",
    "devicelabel",
    "device_label",
    "hostname",
    "rawaudio",
    "raw_audio",
    "audiosamples",
    "audio_samples",
    "address",
    "relatedaddress",
    "related_address",
    "localaddress",
    "local_address",
    "remoteaddress",
    "remote_address",
    "ipaddress",
    "ip_address",
    "signalingurl",
    "signaling_url",
];

const LIVE_FORBIDDEN_PATTERNS: &[&str] = &[
    "candidate:",
    "a=ice",
    "a=fingerprint",
    "c=IN IP",
    "deviceId",
    "groupId",
];

pub fn contains_live_forbidden_content(json: &str) -> Option<String> {
    let lower = json.to_lowercase();
    for pattern in LIVE_FORBIDDEN_PATTERNS {
        if lower.contains(&pattern.to_lowercase()) {
            return Some(format!("forbidden pattern detected: {pattern}"));
        }
    }
    None
}

fn key_is_forbidden(key: &str, path: &str) -> bool {
    if key.eq_ignore_ascii_case("label") && path.contains("dataChannel") {
        return false;
    }
    let key_lower = key.to_lowercase();
    LIVE_FORBIDDEN_KEYS.iter().any(|f| key_lower == *f)
}

pub fn scan_live_value_for_forbidden_keys(value: &Value, path: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if key_is_forbidden(key, path) {
                    return Some(format!("forbidden key at {path}.{key}"));
                }
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if let Some(violation) = scan_live_value_for_forbidden_keys(val, &child_path) {
                    return Some(violation);
                }
            }
            None
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                if let Some(violation) =
                    scan_live_value_for_forbidden_keys(val, &format!("{path}[{i}]"))
                {
                    return Some(violation);
                }
            }
            None
        }
        Value::String(s) => {
            let lower = s.to_lowercase();
            for pattern in LIVE_FORBIDDEN_PATTERNS {
                if lower.contains(pattern) {
                    return Some(format!("forbidden string content at {path}"));
                }
            }
            None
        }
        _ => None,
    }
}
