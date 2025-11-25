use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Standard API error type
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &str {
        match self {
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Unauthorized => "UNAUTHORIZED",
            ApiError::Forbidden => "FORBIDDEN",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_code = self.error_code().to_string();
        let message = self.to_string();

        // Log the error
        tracing::error!(
            error_code = %error_code,
            status = %status_code,
            message = %message,
            "API error occurred"
        );

        let error_response = ErrorResponse {
            code: error_code,
            message,
            details: None,
        };

        (status_code, Json(error_response)).into_response()
    }
}

/// Convenient Result type for API handlers
pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[cfg(test)]
mod tests {
    use super::ApiError;
    use axum::{body, http::StatusCode, response::IntoResponse};
    use serde_json::Value;

    #[tokio::test]
    async fn maps_variants_to_status_and_code() {
        let cases = vec![
            (
                ApiError::NotFound("x".into()),
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
            ),
            (
                ApiError::BadRequest("x".into()),
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
            ),
            (
                ApiError::Unauthorized,
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
            ),
            (ApiError::Forbidden, StatusCode::FORBIDDEN, "FORBIDDEN"),
            (
                ApiError::ValidationError("x".into()),
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
            ),
            (
                ApiError::InternalServerError("x".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
            ),
        ];

        for (err, expected_status, expected_code) in cases {
            let resp = err.into_response();
            let status = resp.status();
            let body = body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(status, expected_status);
            assert_eq!(json.get("code").unwrap(), expected_code);
        }
    }
}
