use crate::data::weather_data::{PressureState, SharedPressureState, SharedWeatherConfig};
use crate::docs::responses::{MessageResponse, PressureResponse};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreatePressureRequest {
    pub air_pressure: f64,
    pub is_running: bool,
}

#[derive(serde::Deserialize, Default, utoipa::ToSchema)]
pub struct UpdatePressureRequest {
    pub air_pressure: Option<f64>,
    pub is_running: Option<bool>,
}

const CALCULATION_INTERVAL_MS: u64 = 100;

#[utoipa::path(
    post,
    path = "/api/pressure",
    tag = "Pressure",
    request_body = CreatePressureRequest,
    responses(
        (status = 201, description = "Sensor tekanan dibuat", body = PressureResponse),
        (status = 409, description = "Config belum lengkap / instance sudah ada", body = MessageResponse)
    )
)]
pub async fn create_pressure(
    data_state: web::Data<SharedPressureState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<CreatePressureRequest>,
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
            .json(serde_json::json!({"message":"Pressure instance already exists."}));
    }

    let req = body.into_inner();
    let new_state = PressureState {
        air_pressure: req.air_pressure,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *guard = Some(new_state.clone());

    HttpResponse::Created()
        .json(serde_json::json!({"message":"Pressure created successfully.", "data": new_state }))
}

#[utoipa::path(
    get,
    path = "/api/pressure",
    tag = "Pressure",
    responses(
        (status = 200, description = "State tekanan", body = PressureResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_pressure(state: web::Data<SharedPressureState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(s) => HttpResponse::Ok().json(serde_json::json!({"message":"Pressure retrieved successfully.", "data": {"air_pressure": s.air_pressure, "is_running": s.is_running, "last_update": s.last_update}})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message":"Pressure data not found"})),
    }
}

#[utoipa::path(
    patch,
    path = "/api/pressure",
    tag = "Pressure",
    request_body = UpdatePressureRequest,
    responses(
        (status = 200, description = "Tekanan diperbarui", body = PressureResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse),
        (status = 409, description = "Config belum lengkap", body = MessageResponse)
    )
)]
pub async fn update_pressure(
    data_state: web::Data<SharedPressureState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<UpdatePressureRequest>,
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
        if let Some(pressure) = patch.air_pressure {
            s.air_pressure = pressure;
        }
        if let Some(run) = patch.is_running {
            s.is_running = run;
        }
        s.last_update = Utc::now();
        HttpResponse::Ok().json(serde_json::json!({"message":"Pressure updated successfully.", "data": {"air_pressure": s.air_pressure, "is_running": s.is_running, "last_update": s.last_update}}))
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({"message":"Pressure data not found to update"}))
    }
}

#[utoipa::path(
    delete,
    path = "/api/pressure",
    tag = "Pressure",
    responses(
        (status = 200, description = "Data dihapus", body = MessageResponse),
        (status = 404, description = "Data belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_pressure(state: web::Data<SharedPressureState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({"message":"Success to delete Pressure data."}))
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({"message":"Pressure running currently not found"}))
    }
}
