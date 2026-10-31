//! Konfigurasi runtime dari environment.
//!
//! Semua nilai punya default yang kompatibel dengan perilaku lama,
//! sehingga `cargo run` tanpa env tetap berjalan seperti sebelumnya.

use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: SocketAddr,
    pub ws_addr: SocketAddr,
    pub tcp_addr: SocketAddr,
    pub mqtt: MqttConfig,
    pub log_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            http_addr: parse_addr("HTTP_ADDR", "127.0.0.1:8080"),
            ws_addr: parse_addr("WS_ADDR", "127.0.0.1:8081"),
            tcp_addr: parse_addr("TCP_ADDR", "127.0.0.1:9001"),
            mqtt: MqttConfig::from_env(),
            log_level: env_or("RUST_LOG", "info"),
        }
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1883,
            client_id: "vessel-client".to_string(),
            username: None,
            password: None,
            keep_alive_secs: 5,
        }
    }
}

impl MqttConfig {
    pub fn from_env() -> Self {
        let defaults = Self::default();
        Self {
            host: env_or("MQTT_HOST", &defaults.host),
            port: env_or("MQTT_PORT", "1883").parse().unwrap_or(defaults.port),
            client_id: env_or("MQTT_CLIENT_ID", &defaults.client_id),
            username: env_opt("MQTT_USERNAME"),
            password: env_opt("MQTT_PASSWORD"),
            keep_alive_secs: env_or("MQTT_KEEP_ALIVE_SECS", "5")
                .parse()
                .unwrap_or(defaults.keep_alive_secs)
                .max(5),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_opt(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

fn parse_addr(key: &str, default: &str) -> SocketAddr {
    let raw = env_or(key, default);
    raw.parse()
        .unwrap_or_else(|e| panic!("{key} tidak valid ({raw}): {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_defaults_are_stable() {
        let cfg = MqttConfig::default();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 1883);
        assert_eq!(cfg.client_id, "vessel-client");
        assert_eq!(cfg.keep_alive_secs, 5);
        assert!(cfg.username.is_none());
        assert!(cfg.password.is_none());
    }
}
