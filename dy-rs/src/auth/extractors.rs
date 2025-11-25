//! Authentication extractors for Axum handlers

use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::{
    config::AuthConfig,
    jwt::{Claims, verify_access_token},
};

fn extract_auth_user_from_parts(parts: &mut Parts) -> Result<AuthUser, AuthError> {
    // Get AuthConfig from extensions (set by middleware)
    let auth_config = parts
        .extensions
        .get::<AuthConfig>()
        .cloned()
        .ok_or_else(|| {
            tracing::error!("AuthConfig not found in extensions. Did you call .with_auth()?");
            AuthError::Internal("Auth not configured".to_string())
        })?;

    // Extract Authorization header
    let auth_header = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Parse Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MissingToken)?;

    // Verify token and extract claims
    let claims = verify_access_token(token, &auth_config).map_err(|_| AuthError::InvalidToken)?;

    Ok(AuthUser::from_claims(claims))
}

/// Authenticated user extracted from JWT token
///
/// Use this extractor in your handlers to require authentication
/// and access user information.
///
/// # Example
///
/// ```rust,ignore
/// use dy_rs::prelude::*;
/// use dy_rs::auth::AuthUser;
///
/// async fn protected_route(user: AuthUser) -> impl IntoResponse {
///     format!("Hello, {}!", user.email)
/// }
///
/// async fn admin_only(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
///     user.require_role("admin")?;
///     Ok("Welcome, admin!")
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// User ID (from JWT subject claim)
    pub id: String,

    /// User email
    pub email: String,

    /// User roles
    pub roles: Vec<String>,

    /// Full JWT claims (for advanced use cases)
    pub claims: Claims,
}

impl AuthUser {
    /// Create AuthUser from JWT claims
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            id: claims.sub.clone(),
            email: claims.email.clone(),
            roles: claims.roles.clone(),
            claims,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Require a specific role, returning an error if not present
    pub fn require_role(&self, role: &str) -> Result<(), AuthError> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!("Role '{}' required", role)))
        }
    }

    /// Require any of the specified roles
    pub fn require_any_role(&self, roles: &[&str]) -> Result<(), AuthError> {
        if self.has_any_role(roles) {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "One of roles {:?} required",
                roles
            )))
        }
    }

    /// Require all of the specified roles
    pub fn require_all_roles(&self, roles: &[&str]) -> Result<(), AuthError> {
        if self.has_all_roles(roles) {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "All of roles {:?} required",
                roles
            )))
        }
    }
}

/// Authentication error type
#[derive(Debug)]
pub enum AuthError {
    /// Missing or invalid Authorization header
    MissingToken,
    /// Invalid or expired token
    InvalidToken,
    /// User lacks required permissions
    Forbidden(String),
    /// Internal error during authentication
    Internal(String),
}

#[derive(Serialize)]
struct AuthErrorResponse {
    code: String,
    message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header missing or invalid".to_string(),
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token".to_string(),
            ),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            AuthError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR", msg),
        };

        let body = AuthErrorResponse {
            code: code.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}

/// State wrapper for AuthConfig - used internally
#[derive(Clone)]
pub struct AuthState {
    pub config: AuthConfig,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move { extract_auth_user_from_parts(parts) }
    }
}

/// Optional authenticated user - doesn't fail if not authenticated
///
/// Useful for routes that behave differently for authenticated vs anonymous users.
///
/// # Example
///
/// ```rust,ignore
/// use dy_rs::auth::OptionalAuthUser;
///
/// async fn maybe_personalized(user: OptionalAuthUser) -> impl IntoResponse {
///     match user.0 {
///         Some(user) => format!("Hello, {}!", user.email),
///         None => "Hello, anonymous!".to_string(),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Reuse the same extraction logic but swallow errors.
            let user = extract_auth_user_from_parts(parts).ok();
            Ok(OptionalAuthUser(user))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::Claims;

    fn mock_claims() -> Claims {
        Claims {
            sub: "user-123".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string(), "editor".to_string()],
            token_type: "access".to_string(),
            iat: 0,
            exp: i64::MAX,
            nbf: 0,
            iss: "test".to_string(),
            aud: "test".to_string(),
            jti: "test-jti".to_string(),
        }
    }

    #[test]
    fn test_auth_user_roles() {
        let user = AuthUser::from_claims(mock_claims());

        assert!(user.has_role("user"));
        assert!(user.has_role("editor"));
        assert!(!user.has_role("admin"));

        assert!(user.has_any_role(&["admin", "user"]));
        assert!(!user.has_any_role(&["admin", "superuser"]));

        assert!(user.has_all_roles(&["user", "editor"]));
        assert!(!user.has_all_roles(&["user", "admin"]));
    }

    #[test]
    fn test_require_role() {
        let user = AuthUser::from_claims(mock_claims());

        assert!(user.require_role("user").is_ok());
        assert!(user.require_role("admin").is_err());
    }
}
