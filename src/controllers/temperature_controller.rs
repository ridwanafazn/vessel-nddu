use actix_web::{web, HttpResponse, Responder};
use crate::data::weather_data::{SharedWeatherConfig, SharedTemperatureState, TemperatureState};
use crate::docs::responses::{MessageResponse, TemperatureResponse};
use chrono::Utc;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateTemperatureRequest {
    pub air_temperature: f64,
    pub is_running: bool,
}

#[derive(serde::Deserialize, Default, utoipa::ToSchema)]
pub struct UpdateTemperatureRequest {
    pub air_temperature: Option<f64>,
    pub is_running: Option<bool>,
}

const CALCULATION_INTERVAL_MS: u64 = 100;

#[utoipa::path(
    post,
    path = "/api/temperature",
    tag = "Temperature",
    request_body = CreateTemperatureRequest,
    responses(
        (status = 201, description = "Sensor suhu dibuat", body = TemperatureResponse),
        (status = 409, description = "Config belum lengkap / instance sudah ada", body = MessageResponse)
    )
)]
pub async fn create_temperature(
    data_state: web::Data<SharedTemperatureState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<CreateTemperatureRequest>,
) -> impl Responder {
    // ensure config is present
    {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({"message":"Cannot create sensor simulation: Configuration is incomplete."}));
        }
    }

    let mut guard = data_state.write().unwrap();
    if guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({"message":"Temperature instance already exists."}));
    }

    let req = body.into_inner();
    let new_state = TemperatureState {
        air_temperature: req.air_temperature,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({"message":"Temperature created successfully.", "data": new_state }))
}

#[utoipa::path(
    get,
    path = "/api/temperature",
    tag = "Temperature",
    responses(
        (status = 200, description = "State suhu", body = TemperatureResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_temperature(state: web::Data<SharedTemperatureState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(s) => HttpResponse::Ok().json(serde_json::json!({"message":"Temperature retrieved successfully.", "data": {"air_temperature": s.air_temperature, "is_running": s.is_running, "last_update": s.last_update}})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message":"Temperature data not found"})),
    }
}

#[utoipa::path(
    patch,
    path = "/api/temperature",
    tag = "Temperature",
    request_body = UpdateTemperatureRequest,
    responses(
        (status = 200, description = "Suhu diperbarui", body = TemperatureResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse),
        (status = 409, description = "Config belum lengkap", body = MessageResponse)
    )
)]
pub async fn update_temperature(
    data_state: web::Data<SharedTemperatureState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<UpdateTemperatureRequest>,
) -> impl Responder {
    let patch = body.into_inner();

    if patch.is_running == Some(true) {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({"message":"Cannot start simulation: Configuration is incomplete."}));
        }
    }

    let mut guard = data_state.write().unwrap();
    if let Some(ref mut s) = *guard {
        if let Some(temp) = patch.air_temperature { s.air_temperature = temp; }
        if let Some(run) = patch.is_running { s.is_running = run; }
        s.last_update = Utc::now();
        HttpResponse::Ok().json(serde_json::json!({"message":"Temperature updated successfully.", "data": {"air_temperature": s.air_temperature, "is_running": s.is_running, "last_update": s.last_update}}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message":"Temperature data not found to update"}))
    }
}

#[utoipa::path(
    delete,
    path = "/api/temperature",
    tag = "Temperature",
    responses(
        (status = 200, description = "Data dihapus", body = MessageResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_temperature(state: web::Data<SharedTemperatureState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({"message":"Success to delete Temperature data."}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message":"Temperature running currently not found"}))
    }
}
