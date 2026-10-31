use actix_web::{web, HttpResponse, Responder};
use crate::controllers::MqttCommandTx;
use crate::data::navigation_data::{
    NavigationConfig, SharedNavigationConfig, UpdateNavigationConfigRequest,
};
use crate::utils::mqtt_manager::MqttCommand;

// === CONFIG HANDLERS ===

pub async fn get_config(config: web::Data<SharedNavigationConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config retrieved successfully.",
        "data": &*guard
    }))
}

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
    guard.enabled_sentences = patch.enabled_sentences.or_else(|| guard.enabled_sentences.clone());

    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config updated successfully.",
        "data": &*guard
    }))
}

pub async fn delete_config(
    config: web::Data<SharedNavigationConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = NavigationConfig::default();

    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigation Config deleted successfully."
    }))
}
