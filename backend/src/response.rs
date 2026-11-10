use axum::Json;
use serde::Serialize;

/// 与旧 Spring Boot 后端保持一致的统一响应体。
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            message: "操作成功".to_string(),
            data: Some(data),
        })
    }

    pub fn success_with_message(message: &str, data: T) -> Json<Self> {
        Json(Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        })
    }

    pub fn success_empty(message: &str) -> Json<ApiResponse<()>> {
        Json(ApiResponse {
            success: true,
            message: message.to_string(),
            data: None,
        })
    }

    pub fn fail(message: &str) -> Json<ApiResponse<()>> {
        Json(ApiResponse {
            success: false,
            message: message.to_string(),
            data: None,
        })
    }
}
