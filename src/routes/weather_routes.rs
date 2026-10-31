use crate::controllers::weather_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/weather")
            // Weather State endpoints
            .route("", web::post().to(weather_controller::create_weather))
            .route("", web::get().to(weather_controller::get_weather))
            .route("", web::patch().to(weather_controller::update_weather))
            .route("", web::delete().to(weather_controller::delete_weather))
            // Weather Config endpoints
            .service(
                web::scope("/config")
                    .route("", web::get().to(weather_controller::get_config))
                    .route("", web::patch().to(weather_controller::post_config))
                    .route("", web::post().to(weather_controller::post_config))
                    .route("", web::delete().to(weather_controller::delete_config)),
            ),
    );
}
