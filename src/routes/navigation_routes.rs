use actix_web::web;
use crate::controllers::navigation_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/navigation")
            .service(
                web::scope("/config")
                    .route("", web::get().to(navigation_controller::get_config))
                    .route("", web::patch().to(navigation_controller::post_config))
                    .route("", web::post().to(navigation_controller::post_config))
                    .route("", web::delete().to(navigation_controller::delete_config)),
            ),
    );
}
