use crate::controllers::humidity_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/humidity")
            .route("", web::post().to(humidity_controller::create_humidity))
            .route("", web::get().to(humidity_controller::get_humidity))
            .route("", web::patch().to(humidity_controller::update_humidity))
            .route("", web::delete().to(humidity_controller::delete_humidity)),
    );
}
