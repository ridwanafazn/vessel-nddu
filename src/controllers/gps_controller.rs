use actix_web::{web, HttpResponse, Responder};
use crate::data::gps_data::{CreateGpsPayload, GpsConfig, GpsData, UpdateGpsConfigPayload, UpdateGpsPayload};
use crate::{AppState, ConfigUpdate};
use crate::utils::gps_calculate;
use chrono::Utc;
use crate::utils::mqtt_manager::MqttManager;

// === CONFIG HANDLERS ===

/// [GET] /api/gps/config
pub async fn get_config(state: web::Data<AppState>) -> impl Responder {
    let guard = state.gps_config.read().await;
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config retrieved successfully.",
        "data": &*guard
    }))
}

/// [PATCH] /api/gps/config
pub async fn update_config(state: web::Data<AppState>, body: web::Json<UpdateGpsConfigPayload>) -> impl Responder {
    let patch = body.into_inner();
    
    let mut test_config = state.gps_config.read().await.clone();
    if let Some(ip) = &patch.ip { test_config.ip = Some(ip.clone()); }
    if let Some(port) = patch.port { test_config.port = Some(port); }
    if let Some(username) = &patch.username { test_config.username = Some(username.clone()); }
    if let Some(password) = &patch.password { test_config.password = Some(password.clone()); }

    if let (Some(ip), Some(port)) = (test_config.ip.clone(), test_config.port) {
        log::info!("[API] Testing new MQTT connection to {}:{}...", ip, port);
        
        let test_result = tokio::task::spawn_blocking(move || {
            MqttManager::test_connection(&ip, port)
        }).await.expect("Task spawn_blocking panik, ini seharusnya tidak terjadi.");

        if let Err(e) = test_result {
            log::warn!("[API] Connection test failed: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": format!("Failed to connect to broker at {}:{}. The port might be closed or the host is unreachable.", test_config.ip.as_deref().unwrap_or_default(), test_config.port.unwrap_or_default()),
                "error": e.to_string()
            }));
        }
        
        log::info!("[API] Connection test successful.");
    }
    
    let mut guard = state.gps_config.write().await;
    if let Some(ip) = patch.ip { guard.ip = Some(ip); }
    if let Some(port) = patch.port { guard.port = Some(port); }
    if let Some(username) = patch.username { guard.username = Some(username); }
    if let Some(password) = patch.password { guard.password = Some(password); }
    if let Some(update_rate) = patch.update_rate { guard.update_rate = Some(update_rate); }
    if let Some(topics) = patch.topics { guard.topics = Some(topics); }

    log::info!("[API] GPS config updated to: {:?}", *guard);
    
    let _ = state.config_update_tx.send(ConfigUpdate::Gps);
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config updated successfully.",
        "data": &*guard
    }))
}

/// [DELETE] /api/gps/config
pub async fn delete_config(state: web::Data<AppState>) -> impl Responder {
    // Langkah 1: Reset konfigurasi ke default.
    *state.gps_config.write().await = GpsConfig::default();
    
    // Langkah 2 (BARU): Hapus juga data sensor yang terkait.
    // .take() akan mengganti nilai di dalam Some(data) menjadi None.
    state.gps_data.write().await.take();
    log::info!("[API] GPS config and associated data have been reset.");

    // Langkah 3: Kirim sinyal agar service di background tahu konfigurasinya sudah hilang.
    let _ = state.config_update_tx.send(ConfigUpdate::Gps);

    // Langkah 4 (BARU): Perbarui pesan respon agar lebih informatif.
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS config and associated sensor data have been reset."
    }))
}

// === SENSOR DATA HANDLERS ===
// (Tidak ada perubahan di fungsi-fungsi di bawah ini)

/// [POST] /api/gps
pub async fn create_gps(state: web::Data<AppState>, body: web::Json<CreateGpsPayload>) -> impl Responder {
    { 
        let config = state.gps_config.read().await;
        if config.ip.is_none() || config.port.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot create sensor data: MQTT configuration (IP, port) is not set."
            }));
        }
    }

    if state.gps_data.read().await.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "GPS data already exists. Please use PATCH to update or DELETE to remove."
        }));
    }

    let req = body.into_inner();

    if !(-90.0..=90.0).contains(&req.latitude) {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid latitude. Must be between -90 and 90."}));
    }
    if !(-180.0..=180.0).contains(&req.longitude) {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid longitude. Must be between -180 and 180."}));
    }
    if req.sog < 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid SOG (Speed Over Ground). Must be non-negative."}));
    }
    if !(0.0..=360.0).contains(&req.cog) {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid COG (Course Over Ground). Must be between 0 and 360."}));
    }

    let now = Utc::now();
    let variation = gps_calculate::calculate_magnetic_variation(req.latitude, req.longitude, &now);
    let new_data = GpsData {
        latitude: req.latitude, longitude: req.longitude,
        sog: req.sog, cog: req.cog,
        is_running: req.is_running,
        variation,
        last_update: now,
    };

    *state.gps_data.write().await = Some(new_data.clone());
    let _ = state.gps_update_tx.send(new_data.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gps created successfully.",
        "data": new_data
    }))
}

/// [GET] /api/gps
pub async fn get_gps(state: web::Data<AppState>) -> impl Responder {
    let guard = state.gps_data.read().await;
    match guard.as_ref() {
        Some(gps_data) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps retrieved successfully.",
            "data": gps_data
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS Data not found" })),
    }
}

/// [PATCH] /api/gps
pub async fn update_gps(state: web::Data<AppState>, body: web::Json<UpdateGpsPayload>) -> impl Responder {
    let patch = body.into_inner();

    if patch.is_running == Some(true) {
        let config = state.gps_config.read().await;
        if config.ip.is_none() || config.port.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot start simulation: MQTT configuration is not set."
            }));
        }
    }

    if let Some(lat) = patch.latitude {
        if !(-90.0..=90.0).contains(&lat) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid latitude."})); }
    }
    if let Some(lon) = patch.longitude {
        if !(-180.0..=180.0).contains(&lon) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid longitude."})); }
    }
    if let Some(sog) = patch.sog {
        if sog < 0.0 { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid SOG."})); }
    }
    if let Some(cog) = patch.cog {
        if !(0.0..=360.0).contains(&cog) { return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid COG."})); }
    }
    
    let mut guard = state.gps_data.write().await;
    if let Some(gps_data) = guard.as_mut() {
        if let Some(lat) = patch.latitude { gps_data.latitude = lat; }
        if let Some(lon) = patch.longitude { gps_data.longitude = lon; }
        if let Some(sog) = patch.sog { gps_data.sog = sog; }
        if let Some(cog) = patch.cog { gps_data.cog = cog; }
        if let Some(is_running) = patch.is_running { gps_data.is_running = is_running; }
        gps_data.last_update = Utc::now();
        
        let updated_data = gps_data.clone();
        drop(guard);
        let _ = state.gps_update_tx.send(updated_data.clone());

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps updated successfully.",
            "data": updated_data
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS Data not found to update" }))
    }
}

/// [DELETE] /api/gps
pub async fn delete_gps(state: web::Data<AppState>) -> impl Responder {
    if state.gps_data.write().await.take().is_some() {
        HttpResponse::Ok().json(serde_json::json!({ "message": "Success to delete GPS data." }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS data not found" }))
    }
}

