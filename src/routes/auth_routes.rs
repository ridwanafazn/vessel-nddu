use actix_web::web;
use crate::controllers::auth_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(auth_controller::login))
    );
}
