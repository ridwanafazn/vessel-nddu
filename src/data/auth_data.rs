use serde::{Deserialize, Serialize};

// Static credentials
pub const ADMIN_USERNAME: &str = "admin";
pub const ADMIN_PASSWORD: &str = "admin";

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}
