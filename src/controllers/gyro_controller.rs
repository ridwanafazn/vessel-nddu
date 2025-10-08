use actix_web::{web, HttpResponse, Responder};
use crate::data::gyro_data::{GyroData, GyroRequest, GyroResponse, GyroConfig};
use crate::services::gyro_service::{self, GyroStore};
use crate::utils::net::Clients;
use crate::utils::mqtt;

/// ==== GYRO SECTION ====

// PATCH GYRO Request (sekarang memakai yaw / yaw_rate)
#[derive(serde::Deserialize)]
pub struct UpdateGyroRequest {
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub yaw_rate: Option<f64>,
    pub is_running: Option<bool>,
}

// PATCH CONFIG Request
#[derive(serde::Deserialize)]
pub struct GyroConfigPatch {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}

pub async fn create_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
    data: web::Json<GyroRequest>,
) -> impl Responder {
    let gyro: GyroData = data.into_inner().into();

    gyro_service::create_gyro(&store, gyro.clone(), Some(&clients), None).await;
    if gyro.is_running {
        gyro_service::start_gyro_stream(store.get_ref().clone(), clients.get_ref().clone(), None);
    }

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gyro created successfully.",
        "data": GyroResponse::from(gyro)
    }))
}

pub async fn get_gyro(store: web::Data<GyroStore>) -> impl Responder {
    match gyro_service::get_gyro(&store).await {
        Some(gyro) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gyro retrieved successfully.",
            "data": GyroResponse::from(gyro)
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}

pub async fn update_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
    data: web::Json<UpdateGyroRequest>,
) -> impl Responder {
    let patch = data.into_inner();
    let mut changed_flag = false;

    let result = gyro_service::update_gyro(
        &store,
        {
            move |gyro| {
                let mut changed = false;
                if let Some(yaw) = patch.yaw {
                    gyro.yaw = yaw;
                    changed = true;
                }
                if let Some(pitch) = patch.pitch {
                    gyro.pitch = pitch;
                    changed = true;
                }
                if let Some(roll) = patch.roll {
                    gyro.roll = roll;
                    changed = true;
                }
                if let Some(rate) = patch.yaw_rate {
                    gyro.yaw_rate = rate;
                    changed = true;
                }
                if let Some(running) = patch.is_running {
                    gyro.is_running = running;
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
        Some(gyro) => {
            if !changed_flag {
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
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Failed to update Gyro Data"
        })),
    }
}

pub async fn delete_gyro(
    store: web::Data<GyroStore>,
    clients: web::Data<Clients>,
) -> impl Responder {
    if gyro_service::delete_gyro(&store, Some(&clients)).await {
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
pub async fn get_config(store: web::Data<GyroStore>) -> impl Responder {
    match gyro_service::get_gyro(&store).await {
        Some(gyro) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Config retrieved successfully.",
            "data": gyro.config
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}

pub async fn update_config(
    store: web::Data<GyroStore>,
    data: web::Json<GyroConfigPatch>,
) -> impl Responder {
    let cfg_patch = data.into_inner();
    let mut changed_flag = false;

    let result = gyro_service::update_gyro(
        &store,
        {
            move |gyro| {
                let mut changed = false;
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
                if changed {
                    changed_flag = true;
                }
            }
        },
        None,
        None,
    ).await;

    match result {
        Some(gyro) => {
            if !changed_flag {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": gyro.config
                }));
            }

            match mqtt::reconnect_if_needed(&gyro.config).await {
                Ok(_) => tracing::info!("MQTT reconnect succeeded with new gyro config"),
                Err(e) => eprintln!("[MQTT] reconnect failed after gyro config patch: {:?}", e),
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

pub async fn delete_config(store: web::Data<GyroStore>) -> impl Responder {
    let result = gyro_service::update_config(
        &store,
        |gyro| {
            gyro.config = GyroConfig::default();
        },
        None,
        None,
    ).await;

    match result {
        Some(gyro) => {
            match mqtt::reconnect_if_needed(&gyro.config).await {
                Ok(_) => tracing::info!("MQTT reconnected to default config for gyro"),
                Err(e) => eprintln!("[MQTT] reconnect failed after gyro config reset: {:?}", e),
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Config reset successfully.",
                "data": gyro.config
            }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Gyro Data not found"
        })),
    }
}
