use actix_web::{web, HttpResponse, Responder};
use crate::controllers::MqttCommandTx; // Import tipe pengirim perintah
use crate::data::gps_data::{
    CreateGpsRequest, GpsConfig, GpsState, SharedGpsConfig, SharedGpsState, UpdateGpsConfigRequest,
    UpdateGpsRequest,
};
use crate::utils::mqtt_manager::MqttCommand;
use crate::utils::gps_calculate;
use chrono::Utc;

// === CONFIG HANDLERS ===

/// [GET] /api/gps/config - Mengambil konfigurasi GPS saat ini.
pub async fn get_config(config: web::Data<SharedGpsConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config retrieved successfully.",
        "data": &*guard
    }))
}

/// [POST] /api/gps/config - Mengisi atau menimpa semua nilai config.
pub async fn post_config(
    config: web::Data<SharedGpsConfig>,
    command_tx: web::Data<MqttCommandTx>,
    body: web::Json<UpdateGpsConfigRequest>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    let patch = body.into_inner();

    // Terapkan semua nilai dari request, gunakan nilai lama jika tidak ada yang baru
    guard.ip = patch.ip.or_else(|| guard.ip.clone());
    guard.port = patch.port.or(guard.port);
    guard.username = patch.username.or_else(|| guard.username.clone());
    guard.password = patch.password.or_else(|| guard.password.clone());
    guard.update_rate = patch.update_rate.or(guard.update_rate);
    guard.topics = patch.topics.or_else(|| guard.topics.clone());

    // Kirim perintah untuk menyambung ulang
    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config updated successfully.",
        "data": &*guard
    }))
}

/// [DELETE] /api/gps/config - Mengosongkan (reset) semua nilai config menjadi null.
pub async fn delete_config(
    config: web::Data<SharedGpsConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = GpsConfig::default(); // Ganti dengan struct default yang semua fieldnya None

    // Kirim perintah untuk menyambung ulang (efektifnya akan memutuskan koneksi)
    let _ = command_tx.send(MqttCommand::Reconnect).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "GPS Config deleted successfully."
    }))
}

// === SENSOR STATE HANDLERS ===

/// [POST] /api/gps - Membuat instance simulasi GPS.
pub async fn create_gps(
    data_state: web::Data<SharedGpsState>,
    config_state: web::Data<SharedGpsConfig>,
    body: web::Json<CreateGpsRequest>,
) -> impl Responder {
    // Validasi: Pastikan config sudah diisi sebelum membuat simulasi
    {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot create sensor simulation: Configuration is incomplete. Please set IP, port, and update_rate."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if data_guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "GPS instance already exists. Please delete it first."
        }));
    }

    let req = body.into_inner();
    let initial_last_update = Utc::now();
    let initial_variation = gps_calculate::calculate_magnetic_variation(req.latitude, req.longitude, &initial_last_update);

    let new_state = GpsState {
        latitude: req.latitude, longitude: req.longitude,
        sog: req.sog, cog: req.cog,
        is_running: req.is_running,
        variation: initial_variation,
        last_update: initial_last_update,
        calculation_rate_ms: 100,
    };

    *data_guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gps created successfully.",
        "data": new_state // Respons sudah bersih, tidak ada config
    }))
}

/// [GET] /api/gps - Mengambil state simulasi GPS saat ini.
pub async fn get_gps(state: web::Data<SharedGpsState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(gps_state) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps retrieved successfully.",
            "data": gps_state
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS Data not found" })),
    }
}

/// [PATCH] /api/gps - Memperbarui sebagian state simulasi GPS.
pub async fn update_gps(
    data_state: web::Data<SharedGpsState>,
    config_state: web::Data<SharedGpsConfig>,
    body: web::Json<UpdateGpsRequest>,
) -> impl Responder {
    let patch = body.into_inner();

    // Validasi: Jika mencoba menyalakan simulasi, pastikan config lengkap
    if patch.is_running == Some(true) {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot start simulation: Configuration is incomplete."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if let Some(ref mut gps_state) = *data_guard {
        if let Some(lat) = patch.latitude { gps_state.latitude = lat; }
        if let Some(lon) = patch.longitude { gps_state.longitude = lon; }
        if let Some(sog) = patch.sog { gps_state.sog = sog; }
        if let Some(cog) = patch.cog { gps_state.cog = cog; }
        if let Some(is_running) = patch.is_running { gps_state.is_running = is_running; }
        gps_state.last_update = Utc::now();

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps updated successfully.",
            "data": gps_state.clone()
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS Data not found to update" }))
    }
}

/// [DELETE] /api/gps - Menghapus instance simulasi GPS.
pub async fn delete_gps(state: web::Data<SharedGpsState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({ "message": "Success to delete GPS live tracking." }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "message": "GPS running currently not found" }))
    }
}