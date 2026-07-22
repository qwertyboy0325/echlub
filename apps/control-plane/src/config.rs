use std::net::SocketAddr;

const DEFAULT_BIND: &str = "127.0.0.1:8080";

#[derive(Debug, Clone)]
pub struct ControlPlaneConfig {
    pub bind_addr: SocketAddr,
    pub extra_origins: Vec<String>,
}

impl ControlPlaneConfig {
    pub fn from_env() -> Result<Self, String> {
        let bind_raw =
            std::env::var("ECHLUB_BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND.to_string());
        let bind_addr: SocketAddr = bind_raw
            .parse()
            .map_err(|_| format!("invalid ECHLUB_BIND_ADDR: {bind_raw}"))?;
        let extra_origins = parse_allowed_origins()?;
        Ok(Self {
            bind_addr,
            extra_origins,
        })
    }
}

pub(crate) fn parse_allowed_origins_from_raw(raw: &str) -> Result<Vec<String>, String> {
    if raw.trim().is_empty() {
        return Err("ECHLUB_ALLOWED_ORIGINS must not be empty when set".into());
    }
    let mut origins = Vec::new();
    for entry in raw.split(',') {
        let origin = entry.trim();
        if origin.is_empty() {
            return Err("ECHLUB_ALLOWED_ORIGINS contains empty entry".into());
        }
        if origin.contains('*') {
            return Err("ECHLUB_ALLOWED_ORIGINS wildcards are forbidden".into());
        }
        if !is_valid_origin(origin) {
            return Err(format!(
                "malformed origin in ECHLUB_ALLOWED_ORIGINS: {origin}"
            ));
        }
        origins.push(origin.to_string());
    }
    origins.sort();
    origins.dedup();
    Ok(origins)
}

fn parse_allowed_origins() -> Result<Vec<String>, String> {
    let Ok(raw) = std::env::var("ECHLUB_ALLOWED_ORIGINS") else {
        return Ok(vec![]);
    };
    parse_allowed_origins_from_raw(&raw)
}

pub fn is_valid_origin(origin: &str) -> bool {
    url::Url::parse(origin).is_ok()
}

pub fn validate_origin(origin: &str, extra_origins: &[String]) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    if host.contains("localhost.") || host.starts_with("127.0.0.1.") {
        return false;
    }
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let port = url
        .port()
        .unwrap_or(if scheme == "https" { 443 } else { 80 });
    if port == 0 {
        return false;
    }
    if (host == "localhost" || host == "127.0.0.1") && (1..=65535).contains(&port) {
        return true;
    }
    extra_origins.iter().any(|o| o == origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;

    #[test]
    fn rejects_wildcard_config() {
        assert!(parse_allowed_origins_from_raw("http://*").is_err());
    }

    #[test]
    fn rejects_empty_allowed_origins_when_set() {
        assert!(parse_allowed_origins_from_raw("   ").is_err());
    }

    #[test]
    fn default_bind_is_loopback() {
        let _lock = test_env::LOCK.lock().unwrap();
        let key = "ECHLUB_ALLOWED_ORIGINS";
        let previous = std::env::var(key).ok();
        std::env::remove_var(key);
        let cfg = ControlPlaneConfig::from_env().unwrap();
        assert_eq!(cfg.bind_addr.to_string(), "127.0.0.1:8080");
        match previous {
            Some(val) => std::env::set_var(key, val),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn rejects_deceptive_localhost_host() {
        assert!(!validate_origin(
            "http://localhost.attacker.example:5173",
            &[]
        ));
    }

    #[test]
    fn accepts_exact_localhost_origin() {
        assert!(validate_origin("http://localhost:5173", &[]));
        assert!(validate_origin("http://127.0.0.1:5173", &[]));
    }

    #[test]
    fn accepts_configured_origin() {
        let extras = vec!["http://192.168.1.10:5173".to_string()];
        assert!(validate_origin("http://192.168.1.10:5173", &extras));
    }
}
