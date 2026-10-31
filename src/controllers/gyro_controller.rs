use actix_web::{web, HttpResponse, Responder};
use crate::controllers::MqttCommandTx;
use crate::docs::responses::{GyroConfigResponse, GyroResponse, MessageResponse};
use crate::data::gyro_data::{
    CreateGyroRequest, GyroConfig, GyroState, SharedGyroConfig, SharedGyroState,
    UpdateGyroConfigRequest, UpdateGyroRequest,
};
use crate::utils::mqtt_manager::MqttCommand;
use chrono::Utc;

// === CONFIG HANDLERS ===

#[allow(clippy::let_and_return)]
#[utoipa::path(
    get,
    path = "/api/gyro/config",
    tag = "Gyro",
    responses((status = 200, description = "Konfigurasi gyro", body = GyroConfigResponse))
)]
pub async fn get_config(config: web::Data<SharedGyroConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config retrieved successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    patch,
    path = "/api/gyro/config",
    tag = "Gyro",
    description = "Perbarui konfigurasi gyro. POST juga diterima sebagai alias.",
    request_body = UpdateGyroConfigRequest,
    responses((status = 200, description = "Config diperbarui", body = GyroConfigResponse))
)]
pub async fn post_config(
    config: web::Data<SharedGyroConfig>,
    command_tx: web::Data<MqttCommandTx>,
    body: web::Json<UpdateGyroConfigRequest>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    let patch = body.into_inner();

    guard.ip = patch.ip.or_else(|| guard.ip.clone());
    guard.port = patch.port.or(guard.port);
    guard.username = patch.username.or_else(|| guard.username.clone());
    guard.password = patch.password.or_else(|| guard.password.clone());
    guard.update_rate = patch.update_rate.or(guard.update_rate);
    guard.topics = patch.topics.or_else(|| guard.topics.clone());
    guard.enabled_sentences = patch.enabled_sentences.or_else(|| guard.enabled_sentences.clone());
    guard.tcp_enabled = patch.tcp_enabled.or(guard.tcp_enabled);
    
    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config updated successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    delete,
    path = "/api/gyro/config",
    tag = "Gyro",
    responses((status = 200, description = "Config di-reset", body = MessageResponse))
)]
pub async fn delete_config(
    config: web::Data<SharedGyroConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = GyroConfig::default();

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config deleted successfully."
    }))
}

// === SENSOR STATE HANDLERS ===

#[utoipa::path(
    post,
    path = "/api/gyro",
    tag = "Gyro",
    request_body = CreateGyroRequest,
    responses(
        (status = 201, description = "Gyro dibuat", body = GyroResponse),
        (status = 409, description = "Config belum lengkap / instance sudah ada", body = MessageResponse)
    )
)]
pub async fn create_gyro(
    data_state: web::Data<SharedGyroState>,
    config_state: web::Data<SharedGyroConfig>,
    body: web::Json<CreateGyroRequest>,
) -> impl Responder {
    {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot create sensor simulation: Configuration is incomplete."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if data_guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "Gyro instance already exists."
        }));
    }

    let req = body.into_inner();
    let new_state = GyroState {
        yaw: req.yaw, pitch: req.pitch, roll: req.roll,
        yaw_rate: req.yaw_rate, is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: 100,
    };

    *data_guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gyro created successfully.",
        "data": new_state
    }))
}

#[utoipa::path(
    get,
    path = "/api/gyro",
    tag = "Gyro",
    responses(
        (status = 200, description = "State gyro", body = GyroResponse),
        (status = 404, description = "Gyro belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_gyro(state: web::Data<SharedGyroState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(gyro_state) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gyro retrieved successfully.",
            "data": gyro_state
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "message": "Gyro Data not found" })),
    }
}

#[utoipa::path(
    patch,
    path = "/api/gyro",
    tag = "Gyro",
    request_body = UpdateGyroRequest,
    responses(
        (status = 200, description = "Gyro diperbarui", body = GyroResponse),
        (status = 404, description = "Gyro belum dibuat", body = MessageResponse),
        (status = 409, description = "Config belum lengkap", body = MessageResponse)
    )
)]
pub async fn update_gyro(
    data_state: web::Data<SharedGyroState>,
    config_state: web::Data<SharedGyroConfig>,
    body: web::Json<UpdateGyroRequest>,
) -> impl Responder {
    let patch = body.into_inner();
    
    if patch.is_running == Some(true) {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot start simulation: Configuration is incomplete."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if let Some(ref mut gyro_state) = *data_guard {
        if let Some(yaw) = patch.yaw { gyro_state.yaw = yaw; }
        if let Some(pitch) = patch.pitch { gyro_state.pitch = pitch; }
        if let Some(roll) = patch.roll { gyro_state.roll = roll; }
        if let Some(yaw_rate) = patch.yaw_rate { gyro_state.yaw_rate = yaw_rate; }
        if let Some(is_running) = patch.is_running { gyro_state.is_running = is_running; }
        gyro_state.last_update = Utc::now();

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Gyro updated successfully.",
            "data": gyro_state.clone()
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "Gyro Data not found to update" }))
    }
}

#[utoipa::path(
    delete,
    path = "/api/gyro",
    tag = "Gyro",
    responses(
        (status = 200, description = "Gyro dihapus", body = MessageResponse),
        (status = 404, description = "Gyro belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_gyro(state: web::Data<SharedGyroState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({ "message": "Success to delete Gyro live tracking." }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "Gyro running currently not found" }))
    }
}