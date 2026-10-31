use crate::controllers::health_controller;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_controller::health));
}
