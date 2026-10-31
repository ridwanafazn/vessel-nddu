use actix_web::web;
use crate::controllers::speed_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/speed")
            .route("", web::post().to(speed_controller::create_speed))
            .route("", web::get().to(speed_controller::get_speed))
            .route("", web::patch().to(speed_controller::update_speed))
            .route("", web::delete().to(speed_controller::delete_speed)),
    );
}
