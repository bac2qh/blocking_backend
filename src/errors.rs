use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Post not found")]
    PostNotFound,

    #[error("Comment not found")]
    CommentNotFound,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Internal server error")]
    InternalError,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::PostNotFound | ApiError::CommentNotFound => StatusCode::NOT_FOUND,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::StorageError(_) | ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({ 
            "error": self.to_string()
        }))
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;