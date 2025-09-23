use actix_web::{web, HttpResponse, Responder};
use crate::data::gps_data::{GPSData, GPSRequest};
use crate::data::gps_data::GPSResponse;
use crate::services::gps_service::{self, GPSStore};
use crate::utils::net::Clients;
use std::panic::AssertUnwindSafe;

#[derive(serde::Deserialize)]
pub struct UpdateGpsRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub speed_over_ground: Option<f64>,
    pub course_over_ground: Option<f64>,
    pub update_rate: Option<u64>,
    pub is_running: Option<bool>,
    pub magnetic_variation: Option<f64>,
}

pub async fn create_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
    data: web::Json<GPSRequest>
) -> impl Responder {
    // Konversi dari request ke state internal GPSData
    let mut gps: GPSData = data.into_inner().into();
    gps.is_running = true;

    gps_service::create_gps(&store, gps.clone(), Some(&clients));
    gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gps created successfully.",
        "data": GPSResponse::from(gps)
    }))
}

pub async fn get_gps(store: web::Data<GPSStore>) -> impl Responder {
    match gps_service::get_gps(&store) {
        Some(gps) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps retrieved successfully.",
            "data": GPSResponse::from(gps)
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS Data not found"
        })),
    }
}

pub async fn update_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
    data: web::Json<UpdateGpsRequest>
) -> impl Responder {
    let mut changed = false;

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        gps_service::update_gps(&store, |gps| {
            if let Some(lat) = data.latitude {
                gps.latitude = lat;
                changed = true;
            }
            if let Some(lon) = data.longitude {
                gps.longitude = lon;
                changed = true;
            }
            if let Some(speed) = data.speed_over_ground {
                gps.speed_over_ground = speed;
                changed = true;
            }
            if let Some(course) = data.course_over_ground {
                gps.course_over_ground = course;
                changed = true;
            }
            if let Some(rate) = data.update_rate {
                gps.update_rate = rate;
                changed = true;
            }
            if let Some(running) = data.is_running {
                gps.is_running = running;
                changed = true;
            }
            if let Some(mag) = data.magnetic_variation {
                gps.magnetic_variation = Some(mag);
                changed = true;
            }
        }, Some(&clients))
    }));

    match result {
        Ok(Some(gps)) => {
            if !changed {
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Nothing changed",
                    "data": GPSResponse::from(gps)
                }));
            }

            if gps.is_running {
                gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone());
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Gps updated successfully.",
                "data": GPSResponse::from(gps)
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Failed to update GPS Data"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Internal server error."
        })),
    }
}

pub async fn delete_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>
) -> impl Responder {
    if gps_service::delete_gps(&store, Some(&clients)) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": "Success to delete GPS live tracking."
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "message": "GPS running currently not found"
        }))
    }
}
