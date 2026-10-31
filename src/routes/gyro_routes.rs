use actix_web::web;
use crate::controllers::gyro_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/gyro")
            .route("", web::post().to(gyro_controller::create_gyro))
            .route("", web::get().to(gyro_controller::get_gyro))
            .route("", web::patch().to(gyro_controller::update_gyro))
            .route("", web::delete().to(gyro_controller::delete_gyro))

            .service(
                web::scope("/config")
                    .route("", web::get().to(gyro_controller::get_config))
                    .route("", web::patch().to(gyro_controller::post_config))
                    .route("", web::post().to(gyro_controller::post_config))
                    .route("", web::delete().to(gyro_controller::delete_config)),
            ),
    );
}