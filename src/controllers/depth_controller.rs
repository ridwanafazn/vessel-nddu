use actix_web::{web, HttpResponse, Responder};
use crate::data::navigation_data::{
    SharedNavigationConfig, SharedDepthState, DepthState,
    CreateDepthRequest, UpdateDepthRequest,
};
use chrono::Utc;

const CALCULATION_INTERVAL_MS: u64 = 100;

pub async fn create_depth(
    data_state: web::Data<SharedDepthState>,
    config_state: web::Data<SharedNavigationConfig>,
    body: web::Json<CreateDepthRequest>,
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
            "message":"Depth instance already exists."
        }));
    }

    let req = body.into_inner();
    let mut new_state = DepthState {
        ocean_water_depth: req.ocean_water_depth,
        constant_clearance: req.constant_clearance,
        under_keel_clearance: 0.0, // will be calculated
        is_running: req.is_running,
        last_update: Utc::now(),
        calculation_rate_ms: CALCULATION_INTERVAL_MS,
    };
    
    // Calculate UKC
    new_state.calculate_ukc();

    *guard = Some(new_state.clone());

    HttpResponse::Created().json(serde_json::json!({
        "message":"Depth created successfully.",
        "data": new_state
    }))
}

pub async fn get_depth(state: web::Data<SharedDepthState>) -> impl Responder {
    let guard = state.read().unwrap();
    match guard.as_ref() {
        Some(depth_state) => HttpResponse::Ok().json(serde_json::json!({
            "message":"Depth retrieved successfully.",
            "data": depth_state
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message":"Depth data not found"
        })),
    }
}

pub async fn update_depth(
    data_state: web::Data<SharedDepthState>,
    config_state: web::Data<SharedNavigationConfig>,
    body: web::Json<UpdateDepthRequest>,
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
    if let Some(ref mut depth_state) = *guard {
        if let Some(depth) = patch.ocean_water_depth {
            depth_state.ocean_water_depth = depth;
        }
        if let Some(clearance) = patch.constant_clearance {
            depth_state.constant_clearance = clearance;
        }
        if let Some(is_running) = patch.is_running {
            depth_state.is_running = is_running;
        }
        
        // Recalculate UKC whenever depth or clearance changes
        depth_state.calculate_ukc();
        depth_state.last_update = Utc::now();

        HttpResponse::Ok().json(serde_json::json!({
            "message":"Depth updated successfully.",
            "data": depth_state.clone()
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message":"Depth data not found to update"
        }))
    }
}

pub async fn delete_depth(state: web::Data<SharedDepthState>) -> impl Responder {
    let mut guard = state.write().unwrap();
    if guard.is_some() {
        *guard = None;
        HttpResponse::Ok().json(serde_json::json!({
            "message":"Success to delete Depth data."
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message":"Depth running currently not found"
        }))
    }
}
