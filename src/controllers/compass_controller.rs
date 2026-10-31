use crate::controllers::MqttCommandTx;
use crate::data::compass_data::{
    CompassConfig, CompassState, SharedCompassConfig, SharedCompassState, COMPASS_DEVIATION,
    COMPASS_VARIATION,
};
use crate::docs::responses::{CompassConfigResponse, CompassResponse, MessageResponse};
use crate::utils::mqtt_manager::MqttCommand;
use actix_web::{web, HttpResponse, Responder};
use chrono::Local;
use serde::Deserialize;

// === REQUEST/RESPONSE MODELS ===

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCompassRequest {
    pub heading: f64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCompassRequest {
    pub heading: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCompassConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub topics: Option<Vec<String>>,
    pub update_rate: Option<u32>,
}

// === CONFIG HANDLERS ===

#[utoipa::path(
    get,
    path = "/api/compass/config",
    tag = "Compass",
    responses((status = 200, description = "Konfigurasi kompas (null jika belum dibuat)", body = CompassConfigResponse))
)]
pub async fn get_config(config: web::Data<SharedCompassConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Compass Config retrieved successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    patch,
    path = "/api/compass/config",
    tag = "Compass",
    description = "Perbarui konfigurasi kompas. POST juga diterima sebagai alias.",
    request_body = UpdateCompassConfigRequest,
    responses((status = 200, description = "Config diperbarui", body = CompassConfigResponse))
)]
pub async fn post_config(
    config: web::Data<SharedCompassConfig>,
    command_tx: web::Data<MqttCommandTx>,
    body: web::Json<UpdateCompassConfigRequest>,
) -> impl Responder {
    let mut guard = config.write().unwrap();

    if guard.is_none() {
        *guard = Some(CompassConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            topics: None,
            update_rate: None,
        });
    }

    let cfg = guard.as_mut().unwrap();
    let patch = body.into_inner();

    cfg.ip = patch.ip.or_else(|| cfg.ip.clone());
    cfg.port = patch.port.or(cfg.port);
    cfg.username = patch.username.or_else(|| cfg.username.clone());
    cfg.password = patch.password.or_else(|| cfg.password.clone());
    cfg.update_rate = patch.update_rate.or(cfg.update_rate);
    cfg.topics = patch.topics.or_else(|| cfg.topics.clone());

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Compass Config updated successfully.",
        "data": cfg
    }))
}

#[utoipa::path(
    delete,
    path = "/api/compass/config",
    tag = "Compass",
    responses((status = 200, description = "Config dihapus", body = MessageResponse))
)]
pub async fn delete_config(
    config: web::Data<SharedCompassConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = None;

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Compass Config deleted successfully."
    }))
}

// === STATE HANDLERS ===

#[utoipa::path(
    get,
    path = "/api/compass",
    tag = "Compass",
    responses(
        (status = 200, description = "State kompas", body = CompassResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_compass(state: web::Data<SharedCompassState>) -> impl Responder {
    let guard = state.read().unwrap();

    match &*guard {
        Some(compass) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Compass data retrieved successfully.",
            "data": {
                "heading": compass.heading,
                "deviation": COMPASS_DEVIATION,
                "variation": COMPASS_VARIATION,
                "is_running": compass.is_running,
                "last_update": compass.last_update,
            }
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Compass data not found."
        })),
    }
}

#[utoipa::path(
    post,
    path = "/api/compass",
    tag = "Compass",
    request_body = CreateCompassRequest,
    responses(
        (status = 201, description = "Kompas dibuat", body = CompassResponse),
        (status = 409, description = "Instance sudah ada", body = MessageResponse)
    )
)]
pub async fn post_compass(
    _config: web::Data<SharedCompassConfig>,
    state: web::Data<SharedCompassState>,
    body: web::Json<CreateCompassRequest>,
) -> impl Responder {
    let mut state_guard = state.write().unwrap();

    if state_guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "Compass instance already exists. Use PATCH to update."
        }));
    }

    let req = body.into_inner();
    let new_state = CompassState {
        heading: req.heading,
        is_running: false, // Default false, bisa diubah dengan PATCH
        last_update: Local::now().format("%d %b %Y, %H:%M:%S").to_string(),
        calculation_rate_ms: 100,
    };

    *state_guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Compass data created successfully.",
        "data": {
            "heading": new_state.heading,
            "deviation": COMPASS_DEVIATION,
            "variation": COMPASS_VARIATION,
            "is_running": new_state.is_running,
            "last_update": new_state.last_update,
        }
    }))
}

#[utoipa::path(
    patch,
    path = "/api/compass",
    tag = "Compass",
    request_body = UpdateCompassRequest,
    responses(
        (status = 200, description = "Kompas diperbarui", body = CompassResponse),
        (status = 400, description = "Config kompas belum lengkap", body = MessageResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn patch_compass(
    config: web::Data<SharedCompassConfig>,
    state: web::Data<SharedCompassState>,
    body: web::Json<UpdateCompassRequest>,
) -> impl Responder {
    let mut guard = state.write().unwrap();

    match guard.as_mut() {
        Some(compass) => {
            let patch = body.into_inner();

            // Validasi config jika akan menjalankan simulasi
            if let Some(true) = patch.is_running {
                let cfg_guard = config.read().unwrap();
                if cfg_guard.is_none() {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Cannot start simulation: Compass Config must be created first."
                    }));
                }
                let cfg = cfg_guard.as_ref().unwrap();
                if !cfg.is_complete() {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Cannot start simulation: Compass Config is incomplete. Required fields: ip, port, username, password, update_rate."
                    }));
                }
            }

            if let Some(heading) = patch.heading {
                compass.heading = heading;
            }

            if let Some(is_running) = patch.is_running {
                compass.is_running = is_running;
            }

            compass.last_update = Local::now().format("%d %b %Y, %H:%M:%S").to_string();

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Compass data updated successfully.",
                "data": {
                    "heading": compass.heading,
                    "deviation": COMPASS_DEVIATION,
                    "variation": COMPASS_VARIATION,
                    "is_running": compass.is_running,
                    "last_update": compass.last_update,
                }
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Compass data not found."
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/api/compass",
    tag = "Compass",
    responses(
        (status = 200, description = "Kompas dihapus", body = MessageResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_compass(state: web::Data<SharedCompassState>) -> impl Responder {
    let mut guard = state.write().unwrap();

    if guard.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "message": "Compass data not found."
        }));
    }

    *guard = None;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Compass data deleted successfully."
    }))
}
