use actix_web::web;
use crate::controllers::pressure_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/pressure")
            .route("", web::post().to(pressure_controller::create_pressure))
            .route("", web::get().to(pressure_controller::get_pressure))
            .route("", web::patch().to(pressure_controller::update_pressure))
            .route("", web::delete().to(pressure_controller::delete_pressure)),
    );
}
