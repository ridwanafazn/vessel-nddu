use actix_web::{web, HttpResponse, Responder};
use crate::controllers::MqttCommandTx;
use crate::data::gyro_data::{
    CreateGyroRequest, GyroConfig, GyroState, SharedGyroConfig, SharedGyroState,
    UpdateGyroConfigRequest, UpdateGyroRequest,
};
use crate::services::MqttCommand;
use chrono::Utc;

// === CONFIG HANDLERS ===

#[allow(clippy::let_and_return)]
pub async fn get_config(config: web::Data<SharedGyroConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config retrieved successfully.",
        "data": &*guard
    }))
}

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
    
    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config updated successfully.",
        "data": &*guard
    }))
}

pub async fn delete_config(
    config: web::Data<SharedGyroConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = GyroConfig::default();

    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config deleted successfully."
    }))
}

// === SENSOR STATE HANDLERS ===

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

pub async fn delete_gyro(state: web::Data<SharedGyroState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({ "message": "Success to delete Gyro live tracking." }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "Gyro running currently not found" }))
    }
}