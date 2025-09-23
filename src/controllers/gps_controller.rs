use actix_web::{web, HttpResponse, Responder};
use crate::data::gps_data::{GPSData, GPSRequest};
use crate::services::gps_service::{self, GPSStore,};
use crate::utils::net::Clients;

#[derive(serde::Deserialize)]
pub struct UpdateGpsRequest {
    pub update_rate: Option<u64>,
    pub running: Option<bool>,
    pub speed_over_ground: Option<f64>,
    pub course_over_ground: Option<f64>,
}

pub async fn create_gps(
    store: web::Data<GPSStore>,
    clients: web::Data<Clients>,
    data: web::Json<GPSRequest>
) -> impl Responder {
    // Konversi dari request ke state internal GPSData
    let mut gps: GPSData = data.into_inner().into();
    gps.running = true;

    gps_service::create_gps(&store, gps.clone(), Some(&clients));
    gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone());

    HttpResponse::Created().json(serde_json::json!({
        "message": "Gps created successfully.",
        "data": gps
    }))
}

pub async fn get_gps(store: web::Data<GPSStore>) -> impl Responder {
    match gps_service::get_gps(&store) {
        Some(gps) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gps retrieved successfully.",
            "data": gps
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
    match gps_service::update_gps(&store, |gps| {
        if let Some(rate) = data.update_rate {
            gps.update_rate = rate;
        }
        if let Some(running) = data.running {
            gps.running = running;
        }
        if let Some(speed) = data.speed_over_ground {
            gps.speed_over_ground = speed;
        }
        if let Some(course) = data.course_over_ground {
            gps.course_over_ground = course;
        }
    }, Some(&clients)) {
        Some(gps) => {
            if gps.running {
                gps_service::start_gps_stream(store.get_ref().clone(), clients.get_ref().clone());
            }
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Gps updated successfully.",
                "data": gps
            }))
        },
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Failed to update GPS Data"
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
