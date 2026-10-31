use crate::controllers::wind_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/wind")
            .route("", web::post().to(wind_controller::create_wind))
            .route("", web::get().to(wind_controller::get_wind))
            .route("", web::patch().to(wind_controller::update_wind))
            .route("", web::delete().to(wind_controller::delete_wind)),
    );
}
