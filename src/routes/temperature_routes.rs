use actix_web::web;
use crate::controllers::temperature_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/temperature")
            .route("", web::post().to(temperature_controller::create_temperature))
            .route("", web::get().to(temperature_controller::get_temperature))
            .route("", web::patch().to(temperature_controller::update_temperature))
            .route("", web::delete().to(temperature_controller::delete_temperature)),
    );
}
