use crate::config::MqttConfig;
use crate::docs::responses::MqttConfigResponse;
use crate::utils::mqtt_manager::MqttManager;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/mqtt/config",
    tag = "MQTT",
    description = "Konfigurasi MQTT global + status koneksi & statistik publish.",
    responses((status = 200, description = "Config MQTT", body = MqttConfigResponse))
)]
pub async fn get_config(manager: web::Data<Arc<MqttManager>>) -> impl Responder {
    let cfg = manager.config();
    let stats = manager.stats();

    HttpResponse::Ok().json(serde_json::json!({
        "message": "MQTT Config retrieved successfully.",
        "data": {
            "host": cfg.host,
            "port": cfg.port,
            "client_id": cfg.client_id,
            "username": cfg.username,
            "keep_alive_secs": cfg.keep_alive_secs,
            "connected": stats.connected(),
            "published": stats.published(),
            "dropped": stats.dropped(),
        }
    }))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateMqttConfigRequest {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive_secs: Option<u64>,
}

#[utoipa::path(
    patch,
    path = "/api/mqtt/config",
    tag = "MQTT",
    description = "Ubah konfigurasi MQTT global. Supervisor akan reconnect otomatis.",
    request_body = UpdateMqttConfigRequest,
    responses((status = 200, description = "Config diperbarui", body = MqttConfigResponse))
)]
pub async fn patch_config(
    manager: web::Data<Arc<MqttManager>>,
    body: web::Json<UpdateMqttConfigRequest>,
) -> impl Responder {
    let mut cfg: MqttConfig = manager.config();
    let patch = body.into_inner();

    if let Some(host) = patch.host {
        cfg.host = host;
    }
    if let Some(port) = patch.port {
        cfg.port = port;
    }
    if let Some(client_id) = patch.client_id {
        cfg.client_id = client_id;
    }
    if let Some(username) = patch.username {
        cfg.username = Some(username);
    }
    if let Some(password) = patch.password {
        cfg.password = Some(password);
    }
    if let Some(keep_alive_secs) = patch.keep_alive_secs {
        cfg.keep_alive_secs = keep_alive_secs.max(5);
    }

    manager.set_config(cfg.clone());

    HttpResponse::Ok().json(serde_json::json!({
        "message": "MQTT Config updated successfully. Reconnecting.",
        "data": {
            "host": cfg.host,
            "port": cfg.port,
            "client_id": cfg.client_id,
            "username": cfg.username,
            "keep_alive_secs": cfg.keep_alive_secs,
        }
    }))
}
