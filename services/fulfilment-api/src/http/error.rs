use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

use crate::http::middleware::RequestId;

/// Standard error response envelope.
///
/// TODO: Decide whether to include a top-level `ok: false` later (avoid churn now).
#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Service-wide API error type.
///
/// Why this exists:
/// - Keeps error responses consistent across handlers.
/// - Makes it easy to add request_id and structured codes everywhere.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub request_id: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn validation(req_id: &RequestId, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "VALIDATION_ERROR",
            message: message.into(),
            request_id: req_id.0.clone(),
            details: None,
        }
    }

    pub fn not_found(req_id: &RequestId) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND",
            message: "Route not found".to_string(),
            request_id: req_id.0.clone(),
            details: None,
        }
    }

    pub fn internal(req_id: &RequestId) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR",
            message: "Internal server error".to_string(),
            request_id: req_id.0.clone(),
            details: None,
        }
    }

    /// Convert a status code into a generic API error, preserving request_id.
    ///
    /// TODO: If we later want richer messages for extractor rejections (e.g., JSON parsing),
    /// introduce a typed mapping rather than parsing arbitrary bodies.
    pub fn from_status(req_id: &RequestId, status: StatusCode) -> Self {
        match status {
            StatusCode::BAD_REQUEST => Self {
                status,
                code: "BAD_REQUEST",
                message: "Bad request".to_string(),
                request_id: req_id.0.clone(),
                details: None,
            },
            StatusCode::NOT_FOUND => Self::not_found(req_id),
            StatusCode::INTERNAL_SERVER_ERROR => Self::internal(req_id),
            _ => Self {
                status,
                code: "ERROR",
                message: "Request failed".to_string(),
                request_id: req_id.0.clone(),
                details: None,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let env = ErrorEnvelope {
            error: ErrorBody {
                code: self.code,
                message: self.message,
                request_id: self.request_id,
                details: self.details,
            },
        };

        (self.status, Json(env)).into_response()
    }
}
