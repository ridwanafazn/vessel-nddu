use actix_web::{web, HttpResponse, Responder};
use crate::data::gps_data::{GPSData, GPSRequest, GPSResponse, GPSConfig};
use crate::services::gps_service::{self, GPSStore};
use crate::utils::net::Clients;

/// ==== GPS SECTION ====

// PATCH GPS Request
#[derive(serde::Deserialize)]
pub struct UpdateGpsRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog: Option<f64>,        // speed over ground
    pub cog: Option<f64>,        // course over ground
    pub update_rate: Option<u64>,
    pub is_running: Option<bool>,
    pub variation: Option<f64>,  // magnetic variation
}

// PATCH CONFIG Request
#[derive(serde::Deserialize)]
pub struct GpsConfigPatch {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}

/// CREATE GPS
pub async fn create_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
    data: web::Json<GPSRequest>,
) -> impl Responder {
    let gps: GPSData = data.into_inner().into();

    gps_service::create_gps(&store, gps.clone(), Some(&clients), None).await;
    if gps.is_running {
        gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone(), None);
    }

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gps created successfully.",
        "data": GPSResponse::from(gps)
    }))
}

/// GET GPS
pub async fn get_gps(store: web::Data<GPSStore>) -> impl Responder {
    match gps_service::get_gps(&store).await {
        Some(gps) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps retrieved successfully.",
            "data": GPSResponse::from(gps)
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS Data not found"
        })),
    }
}

/// UPDATE GPS (PATCH)
pub async fn update_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
    data: web::Json<UpdateGpsRequest>,
) -> impl Responder {
    let patch = data.into_inner();
    let mut changed_flag = false;

    let result = gps_service::update_gps(
        &store,
        {
            let mut changed = false;
            move |gps| {
                if let Some(lat) = patch.latitude {
                    gps.latitude = lat;
                    changed = true;
                }
                if let Some(lon) = patch.longitude {
                    gps.longitude = lon;
                    changed = true;
                }
                if let Some(sog) = patch.sog {
                    gps.sog = sog;
                    changed = true;
                }
                if let Some(cog) = patch.cog {
                    gps.cog = cog;
                    changed = true;
                }
                if let Some(rate) = patch.update_rate {
                    gps.update_rate = rate;
                    gps.config.update_rate = rate;
                    changed = true;
                }
                if let Some(running) = patch.is_running {
                    gps.is_running = running;
                    changed = true;
                }
                if let Some(var) = patch.variation {
                    gps.variation = Some(var);
                    changed = true;
                }
                if changed {
                    changed_flag = true;
                }
            }
        },
        Some(&clients),
        None,
    ).await;

    match result {
        Some(gps) => {
            if !changed_flag {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": GPSResponse::from(gps)
                }));
            }

            if gps.is_running {
                gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone(), None);
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Gps updated successfully.",
                "data": GPSResponse::from(gps)
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Failed to update GPS Data"
        })),
    }
}

/// DELETE GPS
pub async fn delete_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
) -> impl Responder {
    if gps_service::delete_gps(&store, Some(&clients)).await {
        HttpResponse::Ok().json(serde_json::json!({
            "message": "Success to delete GPS live tracking."
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS running currently not found"
        }))
    }
}

/// ==== CONFIG SECTION ====
/// GET CONFIG
pub async fn get_config(store: web::Data<GPSStore>) -> impl Responder {
    match gps_service::get_gps(&store).await {
        Some(gps) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Config retrieved successfully.",
            "data": gps.config
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS Data not found"
        })),
    }
}

/// PATCH CONFIG
pub async fn update_config(
    store: web::Data<GPSStore>,
    data: web::Json<GpsConfigPatch>,
) -> impl Responder {
    let cfg_patch = data.into_inner();
    let mut changed_flag = false;

    let result = gps_service::update_gps(
        &store,
        {
            move |gps| {
                let mut changed = false;
                if let Some(ip) = &cfg_patch.ip {
                    gps.config.ip = ip.clone();
                    changed = true;
                }
                if let Some(port) = cfg_patch.port {
                    gps.config.port = port;
                    changed = true;
                }
                if let Some(username) = &cfg_patch.username {
                    gps.config.username = username.clone();
                    changed = true;
                }
                if let Some(password) = &cfg_patch.password {
                    gps.config.password = password.clone();
                    changed = true;
                }
                if let Some(rate) = cfg_patch.update_rate {
                    gps.config.update_rate = rate;
                    gps.update_rate = rate;
                    changed = true;
                }
                if let Some(topics) = &cfg_patch.topics {
                    gps.config.topics = topics.clone();
                    changed = true;
                }
                if changed {
                    changed_flag = true;
                }
            }
        },
        None,
        None,
    ).await;

    match result {
        Some(gps) => {
            if !changed_flag {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": gps.config
                }));
            }
            HttpResponse::Created().json(serde_json::json!({
                "message": "Config updated successfully.",
                "data": gps.config
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS Data not found"
        })),
    }
}

/// DELETE CONFIG (reset ke default)
pub async fn delete_config(store: web::Data<GPSStore>) -> impl Responder {
    let result = gps_service::update_gps(
        &store,
        |gps| {
            gps.config = GPSConfig::default();
        },
        None,
        None,
    ).await;

    match result {
        Some(gps) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Config reset successfully.",
            "data": gps.config
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS Data not found"
        })),
    }
}
