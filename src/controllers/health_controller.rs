use crate::utils::mqtt_manager::MqttManager;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct AppMeta {
    pub started_at: Instant,
}

impl AppMeta {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Default for AppMeta {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct HealthData {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub mqtt_connected: bool,
    pub mqtt_published: u64,
    pub mqtt_dropped: u64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub data: HealthData,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    description = "Liveness/readiness + status koneksi MQTT.",
    responses((status = 200, description = "OK", body = HealthResponse))
)]
pub async fn health(meta: web::Data<AppMeta>, mqtt: web::Data<Arc<MqttManager>>) -> impl Responder {
    let stats = mqtt.stats();

    HttpResponse::Ok().json(HealthResponse {
        data: HealthData {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: meta.started_at.elapsed().as_secs(),
            mqtt_connected: stats.connected(),
            mqtt_published: stats.published(),
            mqtt_dropped: stats.dropped(),
        },
    })
}
