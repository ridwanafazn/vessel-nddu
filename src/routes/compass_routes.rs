use crate::controllers::compass_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/compass")
            .route("", web::post().to(compass_controller::post_compass))
            .route("", web::get().to(compass_controller::get_compass))
            .route("", web::patch().to(compass_controller::patch_compass))
            .route("", web::delete().to(compass_controller::delete_compass))
            .service(
                web::scope("/config")
                    .route("", web::get().to(compass_controller::get_config))
                    .route("", web::patch().to(compass_controller::post_config))
                    .route("", web::post().to(compass_controller::post_config))
                    .route("", web::delete().to(compass_controller::delete_config)),
            ),
    );
}
