use crate::controllers::mqtt_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mqtt")
            .route("/config", web::get().to(mqtt_controller::get_config))
            .route("/config", web::patch().to(mqtt_controller::patch_config))
            .route("/config", web::post().to(mqtt_controller::patch_config)),
    );
}
