use actix_web::{web, HttpResponse, Responder};
use crate::controllers::MqttCommandTx;
use crate::docs::responses::{MessageResponse, WeatherConfigResponse, WeatherResponse};
use crate::data::weather_data::{
    CreateWeatherRequest, WeatherConfig, WeatherState, SharedWeatherConfig, SharedWeatherState,
    UpdateWeatherConfigRequest, UpdateWeatherRequest,
};
use crate::utils::mqtt_manager::MqttCommand;
use chrono::Utc;

// === CONFIG HANDLERS ===

#[utoipa::path(
    get,
    path = "/api/weather/config",
    tag = "Weather",
    responses((status = 200, description = "Konfigurasi weather", body = WeatherConfigResponse))
)]
pub async fn get_config(config: web::Data<SharedWeatherConfig>) -> impl Responder {
    let guard = config.read().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Weather Config retrieved successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    patch,
    path = "/api/weather/config",
    tag = "Weather",
    description = "Perbarui konfigurasi weather. POST juga diterima sebagai alias.",
    request_body = UpdateWeatherConfigRequest,
    responses((status = 200, description = "Config diperbarui", body = WeatherConfigResponse))
)]
pub async fn post_config(
    config: web::Data<SharedWeatherConfig>,
    command_tx: web::Data<MqttCommandTx>,
    body: web::Json<UpdateWeatherConfigRequest>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    let patch = body.into_inner();

    guard.ip = patch.ip.or_else(|| guard.ip.clone());
    guard.port = patch.port.or(guard.port);
    guard.username = patch.username.or_else(|| guard.username.clone());
    guard.password = patch.password.or_else(|| guard.password.clone());
    guard.update_rate = patch.update_rate.or(guard.update_rate);
    guard.topics = patch.topics.or_else(|| guard.topics.clone());
    guard.enabled_sentences = patch.enabled_sentences.or_else(|| guard.enabled_sentences.clone());

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Weather Config updated successfully.",
        "data": &*guard
    }))
}

#[utoipa::path(
    delete,
    path = "/api/weather/config",
    tag = "Weather",
    responses((status = 200, description = "Config di-reset", body = MessageResponse))
)]
pub async fn delete_config(
    config: web::Data<SharedWeatherConfig>,
    command_tx: web::Data<MqttCommandTx>,
) -> impl Responder {
    let mut guard = config.write().unwrap();
    *guard = WeatherConfig::default();

    let _ = command_tx.try_send(MqttCommand::Reconnect);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Weather Config deleted successfully."
    }))
}

// === SENSOR STATE HANDLERS ===

#[utoipa::path(
    post,
    path = "/api/weather",
    tag = "Weather",
    request_body = CreateWeatherRequest,
    responses(
        (status = 201, description = "Weather dibuat", body = WeatherResponse),
        (status = 409, description = "Config belum lengkap / instance sudah ada", body = MessageResponse)
    )
)]
pub async fn create_weather(
    data_state: web::Data<SharedWeatherState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<CreateWeatherRequest>,
) -> impl Responder {
    {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot create sensor simulation: Configuration is incomplete."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if data_guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "Weather instance already exists."
        }));
    }

    let req = body.into_inner();
    let new_state = WeatherState {
        air_temperature: req.air_temperature,
        air_humidity: req.air_humidity,
        wind_direction_relative: req.wind_direction_relative,
        wind_velocity: req.wind_velocity,
        air_pressure: req.air_pressure,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *data_guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Weather created successfully.",
        "data": new_state
    }))
}

const CALCULATION_INTERVAL_MS: u64 = 100;

#[utoipa::path(
    get,
    path = "/api/weather",
    tag = "Weather",
    responses(
        (status = 200, description = "State weather", body = WeatherResponse),
        (status = 404, description = "Weather belum dibuat", body = MessageResponse)
    )
)]
pub async fn get_weather(state: web::Data<SharedWeatherState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(weather_state) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Weather retrieved successfully.",
            "data": weather_state
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({ 
            "message": "Weather Data not found" 
        })),
    }
}

#[utoipa::path(
    patch,
    path = "/api/weather",
    tag = "Weather",
    request_body = UpdateWeatherRequest,
    responses(
        (status = 200, description = "Weather diperbarui", body = WeatherResponse),
        (status = 404, description = "Weather belum dibuat", body = MessageResponse),
        (status = 409, description = "Config belum lengkap", body = MessageResponse)
    )
)]
pub async fn update_weather(
    data_state: web::Data<SharedWeatherState>,
    config_state: web::Data<SharedWeatherConfig>,
    body: web::Json<UpdateWeatherRequest>,
) -> impl Responder {
    let patch = body.into_inner();

    if patch.is_running == Some(true) {
        let config_guard = config_state.read().unwrap();
        if config_guard.ip.is_none() || config_guard.port.is_none() || config_guard.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Cannot start simulation: Configuration is incomplete."
            }));
        }
    }

    let mut data_guard = data_state.write().unwrap();
    if let Some(ref mut weather_state) = *data_guard {
        // Partial update
        if let Some(temp) = patch.air_temperature { 
            weather_state.air_temperature = temp; 
        }
        if let Some(humidity) = patch.air_humidity { 
            weather_state.air_humidity = humidity; 
        }
        if let Some(wind_dir) = patch.wind_direction_relative { 
            weather_state.wind_direction_relative = wind_dir; 
        }
        if let Some(wind_vel) = patch.wind_velocity { 
            weather_state.wind_velocity = wind_vel; 
        }
        if let Some(pressure) = patch.air_pressure { 
            weather_state.air_pressure = pressure; 
        }
        if let Some(is_running) = patch.is_running { 
            weather_state.is_running = is_running; 
        }
        
        weather_state.last_update = Utc::now();

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Weather updated successfully.",
            "data": weather_state.clone()
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ 
            "message": "Weather Data not found to update" 
        }))
    }
}

#[utoipa::path(
    delete,
    path = "/api/weather",
    tag = "Weather",
    responses(
        (status = 200, description = "Weather dihapus", body = MessageResponse),
        (status = 404, description = "Weather belum dibuat", body = MessageResponse)
    )
)]
pub async fn delete_weather(state: web::Data<SharedWeatherState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({ 
            "message": "Success to delete Weather data." 
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ 
            "message": "Weather running currently not found" 
        }))
    }
}