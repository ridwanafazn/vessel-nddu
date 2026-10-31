use actix_web::{web, HttpResponse, Responder};
use crate::data::auth_data::{LoginRequest, LoginResponse, ADMIN_USERNAME, ADMIN_PASSWORD};

pub async fn login(body: web::Json<LoginRequest>) -> impl Responder {
    let request = body.into_inner();
    
    // Validate credentials against hardcoded values
    if request.username == ADMIN_USERNAME && request.password == ADMIN_PASSWORD {
        HttpResponse::Ok().json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
            username: Some(request.username),
        })
    } else {
        HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            message: "Invalid username or password".to_string(),
            username: None,
        })
    }
}
