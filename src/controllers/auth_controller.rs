use crate::data::auth_data::{LoginRequest, LoginResponse, ADMIN_PASSWORD, ADMIN_USERNAME};
use actix_web::{web, HttpResponse, Responder};
#[allow(unused_imports)]
use serde_json::json; // dipakai di utoipa::path attribute macro

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    summary = "Login ke sistem",
    description = "Login dengan kredensial default.\n\n**Kredensial default:**\n- **username**: `admin`\n- **password**: `admin`\n\n*Catatan: Hanya untuk demo. Production sebaiknya pakai JWT/token.*",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login berhasil", body = LoginResponse,
            example = json!({
                "success": true,
                "message": "Login successful",
                "username": "admin"
            })
        ),
        (status = 401, description = "Kredensial salah", body = LoginResponse,
            example = json!({
                "success": false,
                "message": "Invalid username or password",
                "username": null
            })
        )
    )
)]
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
