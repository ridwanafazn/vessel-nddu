use crate::controllers::MqttCommandTx;
use crate::data::navigation_data::{
    NavigationConfig, SharedNavigationConfig, UpdateNavigationConfigRequest,
};
use crate::docs::responses::{MessageResponse, NavigationConfigResponse};
use crate::utils::mqtt_manager::MqttCommand;
use actix_web::{web, HttpResponse, Responder};

// === CONFIG HANDLERS ===

#[utoipa::path(
    get,
    path = "/api/navigation/config",
    tag = "Navigation",
    responses((status = 200, description = "Konfigurasi navigasi (speed & depth)", body = NavigationConfigResponse))
)]
pub async fn get_config(config: web::Data<SharedNavigationConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config retrieved successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    patch,
    path = "/api/navigation/config",
    tag = "Navigation",
    description = "Perbarui konfigurasi navigasi. POST juga diterima sebagai alias.",
    request_body = UpdateNavigationConfigRequest,
    responses((status = 200, description = "Config diperbarui", body = NavigationConfigResponse))
)]
pub async fn post_config(
    config: web::Data<SharedNavigationConfig>,
    command_tx: web::Data<MqttCommandTx>,
    body: web::Json<UpdateNavigationConfigRequest>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    let patch = body.into_inner();

    guard.ip = patch.ip.or_else(|| guard.ip.clone());
    guard.port = patch.port.or(guard.port);
    guard.username = patch.username.or_else(|| guard.username.clone());
    guard.password = patch.password.or_else(|| guard.password.clone());
    guard.update_rate = patch.update_rate.or(guard.update_rate);
    guard.topics = patch.topics.or_else(|| guard.topics.clone());
    // Update enabled_sentences if present in request
    guard.enabled_sentences = patch
        .enabled_sentences
        .or_else(|| guard.enabled_sentences.clone());

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config updated successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    delete,
    path = "/api/navigation/config",
    tag = "Navigation",
    responses((status = 200, description = "Config di-reset", body = MessageResponse))
)]
pub async fn delete_config(
    config: web::Data<SharedNavigationConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = NavigationConfig::default();

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config deleted successfully."
    }))
}
