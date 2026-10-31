use actix_web::{web, HttpResponse, Responder};
use crate::data::weather_data::{SharedWeatherConfig, SharedHumidityState, HumidityState};
use chrono::Utc;

#[derive(serde::Deserialize)]
pub struct CreateHumidityRequest {
    pub air_humidity: f64,
    pub is_running: bool,
}

#[derive(serde::Deserialize, Default)]
pub struct UpdateHumidityRequest {
    pub air_humidity: Option<f64>,
    pub is_running: Option<bool>,
}

const CALCULATION_INTERVAL_MS: u64 = 100;

pub async fn create_humidity(
    data_state: web::Data<SharedHumidityState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<CreateHumidityRequest>,
) -> impl Responder {
    {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({"message":"Cannot create sensor simulation: Configuration is incomplete."}));
        }
    }

    let mut guard = data_state.write().unwrap();
    if guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({"message":"Humidity instance already exists."}));
    }

    let req = body.into_inner();
    let new_state = HumidityState {
        air_humidity: req.air_humidity,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({"message":"Humidity created successfully.", "data": new_state }))
}

pub async fn get_humidity(state: web::Data<SharedHumidityState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(s) => HttpResponse::Ok().json(serde_json::json!({"message":"Humidity retrieved successfully.", "data": {"air_humidity": s.air_humidity, "is_running": s.is_running, "last_update": s.last_update}})),
        None => HttpResponse::NotFound().json(serde_json::json!({"message":"Humidity data not found"})),
    }
}

pub async fn update_humidity(
    data_state: web::Data<SharedHumidityState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<UpdateHumidityRequest>,
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
        if let Some(humidity) = patch.air_humidity { s.air_humidity = humidity; }
        if let Some(run) = patch.is_running { s.is_running = run; }
        s.last_update = Utc::now();
        HttpResponse::Ok().json(serde_json::json!({"message":"Humidity updated successfully.", "data": {"air_humidity": s.air_humidity, "is_running": s.is_running, "last_update": s.last_update}}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message":"Humidity data not found to update"}))
    }
}

pub async fn delete_humidity(state: web::Data<SharedHumidityState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({"message":"Success to delete Humidity data."}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"message":"Humidity running currently not found"}))
    }
}
