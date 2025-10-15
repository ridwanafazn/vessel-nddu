use actix_web::{web, HttpResponse, Responder};
use crate::data::gyro_data::{CreateGyroPayload, GyroConfig, GyroData, UpdateGyroConfigPayload, UpdateGyroPayload};
use crate::AppState;
use chrono::Utc;
// DIPERBAIKI: Menggunakan path import yang benar untuk BlockingClient.
use rumqttc::{MqttOptions, client::BlockingClient};
use std::time::Duration;


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
    let patch = body.into_inner();
    
    let mut test_config = state.gyro_config.read().await.clone();
    if let Some(ip) = &patch.ip { test_config.ip = Some(ip.clone()); }
    if let Some(port) = patch.port { test_config.port = Some(port); }
    if let Some(username) = &patch.username { test_config.username = Some(username.clone()); }
    if let Some(password) = &patch.password { test_config.password = Some(password.clone()); }

    if let (Some(ip), Some(port)) = (&test_config.ip, test_config.port) {
        log::info!("[API] Testing new MQTT connection to {}:{}...", ip, port);
        let mut mqttoptions = MqttOptions::new("vessel-config-tester-gyro", ip.as_str(), port);
        mqttoptions.set_keep_alive(Duration::from_secs(2));
        if let (Some(user), Some(pass)) = (&test_config.username, &test_config.password) {
            mqttoptions.set_credentials(user, pass);
        }
        
        match BlockingClient::new(mqttoptions, 10) {
            Ok(_) => {
                log::info!("[API] MQTT connection test successful.");
            }
            Err(e) => {
                log::warn!("[API] MQTT connection test failed: {}", e);
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": format!("Failed to connect to MQTT broker at {}:{}.", ip, port),
                    "error": e.to_string()
                }));
            }
        }
    }
    
    let mut guard = state.gyro_config.write().await;
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

// === SENSOR DATA HANDLERS (Tidak ada perubahan di bawah ini) ===

/// [POST] /api/gyro
pub async fn create_gyro(state: web::Data<AppState>, body: web::Json<CreateGyroPayload>) -> impl Responder {
    {
        let config = state.gyro_config.read().await;
        if config.ip.is_none() || config.port.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot create sensor data: MQTT configuration (IP, port) is not set."
            }));
        }
    }
    
    if state.gyro_data.read().await.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({"message": "Gyro data already exists."}));
    }

    let req = body.into_inner();

    if !(-90.0..=90.0).contains(&req.pitch) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid pitch. Must be between -90 and 90."})); }
    if !(-90.0..=90.0).contains(&req.roll) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid roll. Must be between -90 and 90."})); }
    if !(0.0..=360.0).contains(&req.yaw) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid yaw. Must be between 0 and 360."})); }

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
        Some(data) => HttpResponse::Ok().json(serde_json::json!({"message": "Gyro retrieved successfully.", "data": data})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message": "Gyro Data not found"})),
    }
}

/// [PATCH] /api/gyro
pub async fn update_gyro(state: web::Data<AppState>, body: web::Json<UpdateGyroPayload>) -> impl Responder {
    let patch = body.into_inner();

    if patch.is_running == Some(true) {
        let config = state.gyro_config.read().await;
        if config.ip.is_none() || config.port.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot start simulation: MQTT configuration is not set."
            }));
        }
    }

    if let Some(val) = patch.pitch { if !(-90.0..=90.0).contains(&val) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid pitch."})); } }
    if let Some(val) = patch.roll { if !(-90.0..=90.0).contains(&val) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid roll."})); } }
    if let Some(val) = patch.yaw { if !(0.0..=360.0).contains(&val) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid yaw."})); } }

    let mut guard = state.gyro_data.write().await;
    if let Some(data) = guard.as_mut() {
        if let Some(val) = patch.yaw { data.yaw = val; }
        if let Some(val) = patch.pitch { data.pitch = val; }
        if let Some(val) = patch.roll { data.roll = val; }
        if let Some(val) = patch.yaw_rate { data.yaw_rate = val; }
        if let Some(val) = patch.is_running { data.is_running = val; }
        data.last_update = Utc::now();
        let updated_data = data.clone();
        drop(guard);
        let _ = state.gyro_update_tx.send(updated_data.clone());
        
        HttpResponse::Ok().json(serde_json::json!({"message": "Gyro updated successfully.", "data": updated_data}))
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
