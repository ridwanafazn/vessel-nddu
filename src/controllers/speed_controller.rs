use actix_web::{web, HttpResponse, Responder};
use crate::data::navigation_data::{
    SharedNavigationConfig, SharedSpeedState, SpeedState,
    CreateSpeedRequest, UpdateSpeedRequest,
};
use chrono::Utc;

const CALCULATION_INTERVAL_MS: u64 = 100;

pub async fn create_speed(
    data_state: web::Data<SharedSpeedState>,
    config_state: web::Data<SharedNavigationConfig>,
    body: web::Json<CreateSpeedRequest>,
) -> impl Responder {
    {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.username.is_none() || cfg.password.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message":"Cannot create sensor simulation: Configuration is incomplete."
            }));
        }
    }

    let mut guard = data_state.write().unwrap();
    if guard.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message":"Speed instance already exists."
        }));
    }

    let req = body.into_inner();
    let new_state = SpeedState {
        speed_through_water: req.speed_through_water,
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };

    *guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message":"Speed created successfully.",
        "data": new_state
    }))
}

pub async fn get_speed(state: web::Data<SharedSpeedState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(speed_state) => HttpResponse::Ok().json(serde_json::json!({
            "message":"Speed retrieved successfully.",
            "data": speed_state
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message":"Speed data not found"
        })),
    }
}

pub async fn update_speed(
    data_state: web::Data<SharedSpeedState>,
    config_state: web::Data<SharedNavigationConfig>,
    body: web::Json<UpdateSpeedRequest>,
) -> impl Responder {
    let patch = body.into_inner();

    if patch.is_running == Some(true) {
        let cfg = config_state.read().unwrap();
        if cfg.ip.is_none() || cfg.port.is_none() || cfg.username.is_none() || cfg.password.is_none() || cfg.update_rate.is_none() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message":"Cannot start simulation: Configuration is incomplete."
            }));
        }
    }

    let mut guard = data_state.write().unwrap();
    if let Some(ref mut speed_state) = *guard {
        if let Some(speed) = patch.speed_through_water {
            speed_state.speed_through_water = speed;
        }
        if let Some(is_running) = patch.is_running {
            speed_state.is_running = is_running;
        }
        speed_state.last_update = Utc::now();

        HttpResponse::Ok().json(serde_json::json!({
            "message":"Speed updated successfully.",
            "data": speed_state.clone()
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message":"Speed data not found to update"
        }))
    }
}

pub async fn delete_speed(state: web::Data<SharedSpeedState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({
            "message":"Success to delete Speed data."
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message":"Speed running currently not found"
        }))
    }
}
