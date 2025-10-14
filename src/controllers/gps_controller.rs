use actix_web::{web, HttpResponse, Responder};
use crate::data::gps_data::{CreateGpsPayload, GpsConfig, GpsData, UpdateGpsConfigPayload, UpdateGpsPayload};
use crate::AppState;
use crate::utils::gps_calculate;
use chrono::Utc;

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
    let mut guard = state.gps_config.write().await;
    let patch = body.into_inner();
    
    // Terapkan patch. Pola ini lebih aman dan bersih.
    if let Some(ip) = patch.ip { guard.ip = Some(ip); }
    if let Some(port) = patch.port { guard.port = Some(port); }
    if let Some(username) = patch.username { guard.username = Some(username); }
    if let Some(password) = patch.password { guard.password = Some(password); }
    if let Some(update_rate) = patch.update_rate { guard.update_rate = Some(update_rate); }
    if let Some(topics) = patch.topics { guard.topics = Some(topics); }

    log::info!("[API] GPS config updated to: {:?}", *guard);
    
    // DIHAPUS: Tidak perlu lagi mengirim command Reconnect.
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config updated successfully.",
        "data": &*guard
    }))
}

/// [DELETE] /api/gps/config
pub async fn delete_config(state: web::Data<AppState>) -> impl Responder {
    let mut guard = state.gps_config.write().await;
    *guard = GpsConfig::default(); // Reset ke nilai default
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config reset to default."
    }))
}

// === SENSOR DATA HANDLERS ===

/// [POST] /api/gps
pub async fn create_gps(state: web::Data<AppState>, body: web::Json<CreateGpsPayload>) -> impl Responder {
    // Validasi: Pastikan belum ada data GPS yang berjalan
    if state.gps_data.read().await.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "GPS data already exists. Please use PATCH to update or DELETE to remove."
        }));
    }

    let req = body.into_inner();
    let now = Utc::now();
    let variation = gps_calculate::calculate_magnetic_variation(req.latitude, req.longitude, &now);

    let new_data = GpsData {
        latitude: req.latitude, longitude: req.longitude,
        sog: req.sog, cog: req.cog,
        is_running: req.is_running,
        variation,
        last_update: now,
    };

    // Tulis state baru
    *state.gps_data.write().await = Some(new_data.clone());

    // PENTING: Kirim notifikasi update ke semua subscriber agar langsung dipublikasikan.
    if let Err(e) = state.gps_update_tx.send(new_data.clone()) {
        log::warn!("[API] Gagal broadcast data GPS baru: {}", e);
    }

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
    let mut guard = state.gps_data.write().await;
    if let Some(gps_data) = guard.as_mut() {
        let patch = body.into_inner();
        
        // Terapkan patch
        if let Some(lat) = patch.latitude { gps_data.latitude = lat; }
        if let Some(lon) = patch.longitude { gps_data.longitude = lon; }
        if let Some(sog) = patch.sog { gps_data.sog = sog; }
        if let Some(cog) = patch.cog { gps_data.cog = cog; }
        if let Some(is_running) = patch.is_running { gps_data.is_running = is_running; }
        gps_data.last_update = Utc::now();
        
        let updated_data = gps_data.clone();
        drop(guard); // Lepas lock sebelum broadcast

        // Kirim notifikasi update
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
    let mut guard = state.gps_data.write().await;
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({ "message": "Success to delete GPS data." }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS data not found" }))
    }
}