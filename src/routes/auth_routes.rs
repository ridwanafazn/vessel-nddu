use crate::controllers::auth_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/auth").route("/login", web::post().to(auth_controller::login)));
}
