use actix_web::{web, HttpResponse, Responder};
use crate::data::gyro_data::{
    GyroData, GyroRequest, GyroResponse, GyroConfig,
};
use crate::services::gyro_service::{self, GyroStore};
use crate::utils::net::Clients;
use std::panic::AssertUnwindSafe;

/// ==== GYRO SECTION ====

// PATCH GYRO Request
#[derive(serde::Deserialize)]
pub struct UpdateGyroRequest {
    pub heading_true: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub heading_rate: Option<f64>,
    pub is_running: Option<bool>,
}

// PATCH CONFIG Request
#[derive(serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}

/// CREATE GYRO
pub async fn create_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
    data: web::Json<GyroRequest>,
) -> impl Responder {
    let gyro: GyroData = data.into_inner().into();

    gyro_service::create_gyro(&store, gyro.clone(), Some(&clients), None);
    if gyro.is_running {
        gyro_service::start_gyro_stream(store.get_ref().clone(), clients.get_ref().clone(), None);
    }

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gyro created successfully.",
        "data": GyroResponse::from(gyro)
    }))
}

/// GET GYRO
pub async fn get_gyro(store: web::Data<GyroStore>) -> impl Responder {
    match gyro_service::get_gyro(&store) {
        Some(gyro) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gyro retrieved successfully.",
            "data": GyroResponse::from(gyro)
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}

/// UPDATE GYRO (PATCH)
pub async fn update_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
    data: web::Json<UpdateGyroRequest>,
) -> impl Responder {
    let mut changed = false;

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        gyro_service::update_gyro(
            &store,
            |gyro| {
                if let Some(heading) = data.heading_true {
                    gyro.heading_true = heading;
                    changed = true;
                }
                if let Some(pitch) = data.pitch {
                    gyro.pitch = pitch;
                    changed = true;
                }
                if let Some(roll) = data.roll {
                    gyro.roll = roll;
                    changed = true;
                }
                if let Some(rate) = data.heading_rate {
                    gyro.heading_rate = rate;
                    changed = true;
                }
                if let Some(running) = data.is_running {
                    gyro.is_running = running;
                    changed = true;
                }
            },
            Some(&clients),
            None,
        )
    }));

    match result {
        Ok(Some(gyro)) => {
            if !changed {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": GyroResponse::from(gyro)
                }));
            }

            if gyro.is_running {
                gyro_service::start_gyro_stream(store.get_ref().clone(), clients.get_ref().clone(), None);
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Gyro updated successfully.",
                "data": GyroResponse::from(gyro)
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Failed to update Gyro Data"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Internal server error."
        })),
    }
}

/// DELETE GYRO
pub async fn delete_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
) -> impl Responder {
    if gyro_service::delete_gyro(&store, Some(&clients)) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": "Success to delete Gyro live tracking."
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro running currently not found"
        }))
    }
}

/// ==== CONFIG SECTION ====
/// GET CONFIG
pub async fn get_config(store: web::Data<GyroStore>) -> impl Responder {
    match gyro_service::get_gyro(&store) {
        Some(gyro) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Config retrieved successfully.",
            "data": gyro.config // langsung kirim config
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}

/// PATCH CONFIG
pub async fn update_config(
    store: web::Data<GyroStore>,
    data: web::Json<UpdateConfigRequest>,
) -> impl Responder {
    let mut changed = false;
    let cfg_patch = data.into_inner();

    let result = gyro_service::update_gyro(
        &store,
        |gyro| {
            if let Some(ip) = cfg_patch.ip {
                gyro.config.ip = ip;
                changed = true;
            }
            if let Some(port) = cfg_patch.port {
                gyro.config.port = port;
                changed = true;
            }
            if let Some(username) = cfg_patch.username {
                gyro.config.username = username;
                changed = true;
            }
            if let Some(password) = cfg_patch.password {
                gyro.config.password = password;
                changed = true;
            }
            if let Some(rate) = cfg_patch.update_rate {
                gyro.config.update_rate = rate;
                changed = true;
            }
            if let Some(topics) = cfg_patch.topics {
                gyro.config.topics = topics;
                changed = true;
            }
        },
        None,
        None,
    );

    match result {
        Some(gyro) => {
            if !changed {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": gyro.config
                }));
            }
            HttpResponse::Created().json(serde_json::json!({
                "message": "Config updated successfully.",
                "data": gyro.config
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}

/// DELETE CONFIG (reset ke default)
pub async fn delete_config(store: web::Data<GyroStore>) -> impl Responder {
    let result = gyro_service::update_gyro(
        &store,
        |gyro| {
            gyro.config = GyroConfig {
                ip: "127.0.0.1".to_string(),
                port: 1883,
                username: "guest".to_string(),
                password: "guest".to_string(),
                update_rate: 1000,
                topics: vec!["gyro/default".to_string()],
            };
        },
        None,
        None,
    );

    match result {
        Some(gyro) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Config reset successfully.",
            "data": gyro.config
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}
