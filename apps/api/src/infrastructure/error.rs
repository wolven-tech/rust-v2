//! The single error type every handler returns.
//!
//! Having exactly one gives `rv2-client` exactly one error body to parse, and
//! makes "which status does this failure produce?" answerable by reading one
//! `match` instead of grepping handlers.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rv2_api_types::ErrorResponse;
use rv2_domain::ValidationError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{kind} not found")]
    NotFound { kind: &'static str },

    #[error("not authenticated")]
    Unauthenticated,

    /// Authenticated but not permitted. Deliberately distinct from
    /// [`ApiError::Unauthenticated`]: the client redirects to `/login` on 401
    /// and would loop forever if 403 were reported the same way.
    #[error("not permitted")]
    Forbidden,

    #[error("too many requests")]
    RateLimited,

    #[error("event store error: {0}")]
    Store(#[from] rv2_allsource::AppendError),

    #[error("event store error: {0}")]
    Sdk(#[from] rv2_allsource::SdkError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    fn parts(&self) -> (StatusCode, &'static str) {
        match self {
            ApiError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_failed"),
            ApiError::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::Unauthenticated => (StatusCode::UNAUTHORIZED, "unauthorized"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            ApiError::Store(_) | ApiError::Sdk(_) => (StatusCode::BAD_GATEWAY, "store_unavailable"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = self.parts();

        // 5xx bodies are deliberately generic: an internal error message can
        // carry connection strings or stack detail. The real text goes to the
        // log, keyed by the same status, not to the client.
        let message = if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
            "an internal error occurred".to_string()
        } else {
            self.to_string()
        };

        (
            status,
            Json(ErrorResponse {
                code: code.to_string(),
                message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_match_what_the_client_expects() {
        assert_eq!(
            ApiError::Unauthenticated.parts().0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(ApiError::Forbidden.parts().0, StatusCode::FORBIDDEN);
        assert_eq!(
            ApiError::NotFound { kind: "post" }.parts().0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::Validation(ValidationError::new("title", "x"))
                .parts()
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    /// 401 and 403 must not collapse into one status: `rv2-client` redirects to
    /// `/login` on 401, and doing that on 403 produces a redirect loop.
    #[test]
    fn unauthenticated_and_forbidden_are_distinct() {
        assert_ne!(
            ApiError::Unauthenticated.parts(),
            ApiError::Forbidden.parts()
        );
    }

    #[test]
    fn error_codes_are_stable_machine_readable_strings() {
        assert_eq!(ApiError::RateLimited.parts().1, "rate_limited");
        assert_eq!(ApiError::Internal("x".into()).parts().1, "internal_error");
    }
}
