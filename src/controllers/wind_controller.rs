use crate::data::weather_data::{SharedWeatherConfig, SharedWindState, WindState};
use crate::docs::responses::{MessageResponse, WindResponse};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateWindRequest {
    pub wind_direction_relative: f64,
    pub wind_velocity: f64,
    pub is_running: bool,
}

#[derive(serde::Deserialize, Default, utoipa::ToSchema)]
pub struct UpdateWindRequest {
    pub wind_direction_relative: Option<f64>,
    pub wind_velocity: Option<f64>,
    pub is_running: Option<bool>,
}

const CALCULATION_INTERVAL_MS: u64 = 100;

#[utoipa::path(
    post,
    path = "/api/wind",
    tag = "Wind",
    request_body = CreateWindRequest,
    responses(
        (status = 201, description = "Sensor angin dibuat", body = WindResponse),
        (status = 409, description = "Config belum lengkap / instance sudah ada", body = MessageResponse)
    )
)]
pub async fn create_wind(
    data_state: web::Data<SharedWindState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<CreateWindRequest>,
) -> impl Responder {
    {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({"message":"Cannot create sensor simulation: Configuration is incomplete."}));
        }
    }

    let mut guard = data_state.write().unwrap();
    if guard.is_some() {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"message":"Wind instance already exists."}));
    }

    let req = body.into_inner();
    let new_state = WindState {
        wind_direction_relative: req.wind_direction_relative,
        wind_velocity: req.wind_velocity,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *guard = Some(new_state.clone());

    HttpResponse::Created()
        .json(serde_json::json!({"message":"Wind created successfully.", "data": new_state }))
}

#[utoipa::path(
    get,
    path = "/api/wind",
    tag = "Wind",
    responses(
        (status = 200, description = "State angin", body = WindResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_wind(state: web::Data<SharedWindState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(s) => HttpResponse::Ok().json(serde_json::json!({"message":"Wind retrieved successfully.", "data": {"wind_direction_relative": s.wind_direction_relative, "wind_velocity": s.wind_velocity, "is_running": s.is_running, "last_update": s.last_update}})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message":"Wind data not found"})),
    }
}

#[utoipa::path(
    patch,
    path = "/api/wind",
    tag = "Wind",
    request_body = UpdateWindRequest,
    responses(
        (status = 200, description = "Angin diperbarui", body = WindResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse),
        (status = 409, description = "Config belum lengkap", body = MessageResponse)
    )
)]
pub async fn update_wind(
    data_state: web::Data<SharedWindState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<UpdateWindRequest>,
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
        if let Some(dir) = patch.wind_direction_relative {
            s.wind_direction_relative = dir;
        }
        if let Some(vel) = patch.wind_velocity {
            s.wind_velocity = vel;
        }
        if let Some(run) = patch.is_running {
            s.is_running = run;
        }
        s.last_update = Utc::now();
        HttpResponse::Ok().json(serde_json::json!({"message":"Wind updated successfully.", "data": {"wind_direction_relative": s.wind_direction_relative, "wind_velocity": s.wind_velocity, "is_running": s.is_running, "last_update": s.last_update}}))
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({"message":"Wind data not found to update"}))
    }
}

#[utoipa::path(
    delete,
    path = "/api/wind",
    tag = "Wind",
    responses(
        (status = 200, description = "Data dihapus", body = MessageResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_wind(state: web::Data<SharedWindState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({"message":"Success to delete Wind data."}))
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({"message":"Wind running currently not found"}))
    }
}
