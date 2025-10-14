use actix_web::{web, HttpResponse, Responder};
use crate::data::gyro_data::{CreateGyroPayload, GyroConfig, GyroData, UpdateGyroConfigPayload, UpdateGyroPayload};
use crate::AppState;
use chrono::Utc;

// === CONFIG HANDLERS ===

/// [GET] /api/gyro/config
pub async fn get_config(state: web::Data<AppState>) -> impl Responder {
    let guard = state.gyro_config.read().await;
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config retrieved successfully.",
        "data": &*guard
    }))
}

/// [PATCH] /api/gyro/config
pub async fn update_config(state: web::Data<AppState>, body: web::Json<UpdateGyroConfigPayload>) -> impl Responder {
    let mut guard = state.gyro_config.write().await;
    let patch = body.into_inner();
    if let Some(ip) = patch.ip { guard.ip = Some(ip); }
    if let Some(port) = patch.port { guard.port = Some(port); }
    if let Some(username) = patch.username { guard.username = Some(username); }
    if let Some(password) = patch.password { guard.password = Some(password); }
    if let Some(update_rate) = patch.update_rate { guard.update_rate = Some(update_rate); }
    if let Some(topics) = patch.topics { guard.topics = Some(topics); }
    log::info!("[API] Gyro config updated to: {:?}", *guard);
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Gyro Config updated successfully.",
        "data": &*guard
    }))
}

/// [DELETE] /api/gyro/config
pub async fn delete_config(state: web::Data<AppState>) -> impl Responder {
    *state.gyro_config.write().await = GyroConfig::default();
    HttpResponse::Ok().json(serde_json::json!({"message": "Gyro Config reset to default."}))
}

// === SENSOR DATA HANDLERS ===
// (Pola yang sama persis seperti gps_controller)

/// [POST] /api/gyro
pub async fn create_gyro(state: web::Data<AppState>, body: web::Json<CreateGyroPayload>) -> impl Responder {
    if state.gyro_data.read().await.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "Gyro data already exists."
        }));
    }
    let req = body.into_inner();
    let new_data = GyroData {
        yaw: req.yaw, pitch: req.pitch, roll: req.roll,
        yaw_rate: req.yaw_rate,
        is_running: req.is_running,
        last_update: Utc::now(),
    };
    *state.gyro_data.write().await = Some(new_data.clone());
    let _ = state.gyro_update_tx.send(new_data.clone());
    HttpResponse::Created().json(serde_json::json!({
        "message": "Gyro created successfully.",
        "data": new_data
    }))
}

/// [GET] /api/gyro
pub async fn get_gyro(state: web::Data<AppState>) -> impl Responder {
    let guard = state.gyro_data.read().await;
    match guard.as_ref() {
        Some(data) => HttpResponse::Ok().json(serde_json::json!({"data": data})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message": "Gyro Data not found"})),
    }
}

/// [PATCH] /api/gyro
pub async fn update_gyro(state: web::Data<AppState>, body: web::Json<UpdateGyroPayload>) -> impl Responder {
    let mut guard = state.gyro_data.write().await;
    if let Some(data) = guard.as_mut() {
        let patch = body.into_inner();
        if let Some(val) = patch.yaw { data.yaw = val; }
        if let Some(val) = patch.pitch { data.pitch = val; }
        if let Some(val) = patch.roll { data.roll = val; }
        if let Some(val) = patch.yaw_rate { data.yaw_rate = val; }
        if let Some(val) = patch.is_running { data.is_running = val; }
        data.last_update = Utc::now();
        let updated_data = data.clone();
        drop(guard);
        let _ = state.gyro_update_tx.send(updated_data.clone());
        HttpResponse::Ok().json(serde_json::json!({"data": updated_data}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message": "Gyro Data not found to update"}))
    }
}

/// [DELETE] /api/gyro
pub async fn delete_gyro(state: web::Data<AppState>) -> impl Responder {
    if state.gyro_data.write().await.take().is_some() {
        HttpResponse::Ok().json(serde_json::json!({"message": "Success to delete Gyro data."}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message": "Gyro data not found"}))
    }
}