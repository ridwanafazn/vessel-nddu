use actix_web::web;
use crate::controllers::gps_controller;

/// Initialize GPS routes under `/api/gps`
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/gps")
            // R6 endpoint (CRUD)
            .route("/r6", web::post().to(gps_controller::create_gps))
            .route("/r6", web::get().to(gps_controller::get_gps))
            .route("/r6", web::patch().to(gps_controller::update_gps))
            .route("/r6", web::delete().to(gps_controller::delete_gps))
    );
}
