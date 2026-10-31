use crate::controllers::depth_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/depth")
            .route("", web::post().to(depth_controller::create_depth))
            .route("", web::get().to(depth_controller::get_depth))
            .route("", web::patch().to(depth_controller::update_depth))
            .route("", web::delete().to(depth_controller::delete_depth)),
    );
}
