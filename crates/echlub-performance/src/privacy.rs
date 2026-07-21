use serde_json::Value;

const FORBIDDEN_KEYS: &[&str] = &[
    "sdp",
    "ice",
    "candidate",
    "deviceid",
    "device_id",
    "groupid",
    "group_id",
    "label",
    "ip",
    "ipaddress",
    "ip_address",
    "localip",
    "local_ip",
    "remoteip",
    "remote_ip",
    "fingerprint",
    "ssrc",
];

fn key_is_forbidden(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    FORBIDDEN_KEYS.iter().any(|f| key_lower == *f)
}

const FORBIDDEN_PATTERNS: &[&str] = &[
    "candidate:",
    "a=ice",
    "a=fingerprint",
    "c=IN IP",
    "deviceId",
    "groupId",
];

pub fn contains_forbidden_content(json: &str) -> Option<String> {
    let lower = json.to_lowercase();
    for pattern in FORBIDDEN_PATTERNS {
        if lower.contains(&pattern.to_lowercase()) {
            return Some(format!("forbidden pattern detected: {pattern}"));
        }
    }
    None
}

pub fn scan_value_for_forbidden_keys(value: &Value, path: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if key_is_forbidden(key) {
                    return Some(format!("forbidden key at {path}.{key}"));
                }
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if let Some(violation) = scan_value_for_forbidden_keys(val, &child_path) {
                    return Some(violation);
                }
            }
            None
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                if let Some(violation) = scan_value_for_forbidden_keys(val, &format!("{path}[{i}]"))
                {
                    return Some(violation);
                }
            }
            None
        }
        Value::String(s) => {
            let lower = s.to_lowercase();
            for pattern in FORBIDDEN_PATTERNS {
                if lower.contains(&pattern.to_lowercase()) {
                    return Some(format!("forbidden string content at {path}"));
                }
            }
            None
        }
        _ => None,
    }
}
