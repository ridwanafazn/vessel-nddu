use actix_web::web;
use crate::controllers::gps_controller;

/// Initialize GPS + Config routes under `/api/gps`
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/gps")
            // GPS CRUD
            .route("", web::post().to(gps_controller::create_gps))
            .route("", web::get().to(gps_controller::get_gps))
            .route("", web::patch().to(gps_controller::update_gps))
            .route("", web::delete().to(gps_controller::delete_gps))
            // Config CRUD (nested under /api/gps/config)
            .service(
                web::scope("/config")
                    .route("", web::get().to(gps_controller::get_config))
                    .route("", web::patch().to(gps_controller::update_config))
                    .route("", web::delete().to(gps_controller::delete_config)),
            ),
    );
}
