use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;

use crate::response::ApiResponse;

/// 业务异常与应用异常统一封装。
/// 业务错误返回 HTTP 200 + success=false，保持旧客户端解析逻辑不变；
/// 鉴权失败与越权访问使用 401/403，但响应体结构不变。
#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub status: StatusCode,
}

impl AppError {
    pub fn business(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::OK,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
        }
    }

    pub fn bad_request() -> Self {
        Self::business("请求参数格式错误")
    }

    pub fn internal() -> Self {
        Self::business("系统繁忙，请稍后重试")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            ApiResponse::<()>::fail(&self.message),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(error = %error, "数据库操作失败");
        Self::internal()
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        tracing::error!(error = %error, "文件操作失败");
        Self::business("图片处理失败，请稍后重试")
    }
}

impl From<JsonRejection> for AppError {
    fn from(_: JsonRejection) -> Self {
        Self::bad_request()
    }
}

/// 将 JSON 解析失败也转换成统一响应体，避免返回纯文本 400。
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(AppError::from)?;
        Ok(Self(value))
    }
}
