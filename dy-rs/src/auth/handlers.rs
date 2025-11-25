//! Authentication route handlers

use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};

use super::{
    config::AuthConfig,
    extractors::AuthUser,
    jwt::{create_token_pair, verify_refresh_token},
    models::*,
};
use crate::error::ApiError;
use crate::extractors::ValidatedJson;

/// User storage trait - implement this for your database
///
/// This trait defines the interface for user storage operations.
/// Implement this for your specific database (PostgreSQL, MySQL, etc.)
///
/// # Example
///
/// ```rust,ignore
/// use dy_rs::auth::{UserStore, StoredUser};
/// use sqlx::PgPool;
///
/// struct PostgresUserStore {
///     pool: PgPool,
/// }
///
/// #[async_trait]
/// impl UserStore for PostgresUserStore {
///     async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
///         let user = sqlx::query_as!(
///             StoredUser,
///             "SELECT id, email, name, password_hash, roles FROM users WHERE email = $1",
///             email
///         )
///         .fetch_optional(&self.pool)
///         .await?;
///         Ok(user)
///     }
///     
///     // ... implement other methods
/// }
/// ```
#[async_trait::async_trait]
pub trait UserStore: Send + Sync + 'static {
    /// Find a user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError>;

    /// Find a user by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError>;

    /// Create a new user
    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError>;

    /// Update user's password hash
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError>;

    /// Check if email is already taken
    async fn email_exists(&self, email: &str) -> Result<bool, ApiError>;
}

/// Stored user data from database
#[derive(Debug, Clone)]
pub struct StoredUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub roles: Vec<String>,
}

/// Data for creating a new user
#[derive(Debug, Clone)]
pub struct CreateUserData {
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

/// In-memory user store for development/testing
///
/// **WARNING: Do not use in production!**
/// This is only for development and testing purposes.
#[derive(Clone, Default)]
pub struct InMemoryUserStore {
    users: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, StoredUser>>>,
}

impl InMemoryUserStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl UserStore for InMemoryUserStore {
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(id).cloned())
    }

    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError> {
        let mut users = self.users.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let stored = StoredUser {
            id: id.clone(),
            email: user.email,
            name: user.name,
            password_hash: user.password_hash,
            roles: vec!["user".to_string()],
        };
        users.insert(id, stored.clone());
        Ok(stored)
    }

    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError> {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get_mut(id) {
            user.password_hash = password_hash.to_string();
            Ok(())
        } else {
            Err(ApiError::NotFound("User not found".to_string()))
        }
    }

    async fn email_exists(&self, email: &str) -> Result<bool, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().any(|u| u.email == email))
    }
}

/// Application state for auth routes
#[derive(Clone)]
pub struct AuthAppState<S: UserStore> {
    pub config: AuthConfig,
    pub user_store: S,
}

/// Login handler
///
/// Authenticates a user with email and password, returns JWT tokens.
pub async fn login<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Find user by email
    let user = state
        .user_store
        .find_by_email(&payload.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;

    // Verify password
    let password_valid = super::password::verify_password(&payload.password, &user.password_hash)?;
    if !password_valid {
        return Err(ApiError::Unauthorized);
    }

    // Generate tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;

    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Registration handler
///
/// Creates a new user account and returns JWT tokens.
pub async fn register<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validate password strength
    super::password::validate_password_strength(&payload.password)?;

    // Check if email is already taken
    if state.user_store.email_exists(&payload.email).await? {
        return Err(ApiError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = super::password::hash_password(&payload.password, &state.config)?;

    // Create user
    let user = state
        .user_store
        .create(CreateUserData {
            email: payload.email,
            name: payload.name,
            password_hash,
        })
        .await?;

    // Generate tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;

    tracing::info!(user_id = %user.id, "New user registered");

    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Refresh token handler
///
/// Exchanges a refresh token for a new access/refresh token pair.
pub async fn refresh_token<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<TokenRefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Verify refresh token
    let claims = verify_refresh_token(&payload.refresh_token, &state.config)?;

    // Get user (to ensure they still exist and get current roles)
    let user = state
        .user_store
        .find_by_id(&claims.sub)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;

    // Generate new tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;

    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Logout handler
///
/// For stateless JWT, this is a no-op on the server side.
/// In a production app, you might want to:
/// - Add the token to a blacklist
/// - Invalidate the refresh token in the database
pub async fn logout() -> Json<MessageResponse> {
    // For stateless JWT, logout is handled client-side by discarding tokens
    // In production, you might want to blacklist the token or invalidate refresh tokens
    Json(MessageResponse::new("Successfully logged out"))
}

/// Get current user info
pub async fn me<S: UserStore>(
    user: AuthUser,
    State(state): State<AuthAppState<S>>,
) -> Result<Json<AuthUserInfo>, ApiError> {
    let stored_user = state
        .user_store
        .find_by_id(&user.id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(AuthUserInfo {
        id: stored_user.id,
        email: stored_user.email,
        name: stored_user.name,
        roles: stored_user.roles,
    }))
}

/// Create auth routes with a custom user store
///
/// # Example
///
/// ```rust,ignore
/// use dy_rs::auth::{auth_routes_with_store, AuthConfig, InMemoryUserStore};
///
/// let config = AuthConfig::default();
/// let store = InMemoryUserStore::new();
///
/// let routes = auth_routes_with_store(config, store);
/// ```
pub fn auth_routes_with_store<S: UserStore + Clone>(config: AuthConfig, user_store: S) -> Router {
    let state = AuthAppState {
        config: config.clone(),
        user_store,
    };

    Router::new()
        .route("/auth/login", post(login::<S>))
        .route("/auth/register", post(register::<S>))
        .route("/auth/refresh", post(refresh_token::<S>))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me::<S>))
        .with_state(state)
}

/// Create auth routes with in-memory store (for development)
///
/// **WARNING: Do not use in production!**
pub fn auth_routes(config: AuthConfig) -> Router {
    auth_routes_with_store(config, InMemoryUserStore::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        middleware::Next,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let config = AuthConfig::default();
        let routes = auth_routes_with_store(config.clone(), InMemoryUserStore::new());
        routes.layer(middleware::from_fn(
            move |mut req: Request<Body>, next: Next| {
                let cfg = config.clone();
                async move {
                    req.extensions_mut().insert(cfg);
                    next.run(req).await
                }
            },
        ))
    }

    fn json_req(uri: &str, body: &Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn register_then_me_returns_user_info() {
        let app = test_app();
        let payload = serde_json::json!({
            "email": "user@example.com",
            "password": "StrongPass1",
            "name": "User"
        });

        let res = app
            .clone()
            .oneshot(json_req("/auth/register", &payload))
            .await
            .expect("register request should succeed");
        assert_eq!(res.status(), StatusCode::OK);
        let body: AuthResponse =
            serde_json::from_slice(&to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();

        let me_req = Request::builder()
            .method("GET")
            .uri("/auth/me")
            .header("authorization", format!("Bearer {}", body.access_token))
            .body(Body::empty())
            .unwrap();

        let me_res = app
            .clone()
            .oneshot(me_req)
            .await
            .expect("me request should succeed");
        assert_eq!(me_res.status(), StatusCode::OK);
        let user: AuthUserInfo =
            serde_json::from_slice(&to_bytes(me_res.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(user.email, "user@example.com");
        assert_eq!(user.name, "User");
        assert_eq!(user.roles, vec!["user".to_string()]);
    }

    #[tokio::test]
    async fn login_and_refresh_flow() {
        let app = test_app();
        // Register first
        let register_payload = serde_json::json!({
            "email": "login@example.com",
            "password": "StrongPass1",
            "name": "Login"
        });
        let _ = app
            .clone()
            .oneshot(json_req("/auth/register", &register_payload))
            .await
            .unwrap();

        // Login
        let login_payload = serde_json::json!({
            "email": "login@example.com",
            "password": "StrongPass1"
        });
        let login_res = app
            .clone()
            .oneshot(json_req("/auth/login", &login_payload))
            .await
            .unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);
        let login_body: AuthResponse =
            serde_json::from_slice(&to_bytes(login_res.into_body(), usize::MAX).await.unwrap())
                .unwrap();

        // Refresh
        let refresh_payload = serde_json::json!({
            "refresh_token": login_body.refresh_token
        });
        let refresh_res = app
            .oneshot(json_req("/auth/refresh", &refresh_payload))
            .await
            .unwrap();
        assert_eq!(refresh_res.status(), StatusCode::OK);
        let refreshed: AuthResponse =
            serde_json::from_slice(&to_bytes(refresh_res.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(refreshed.user.email, "login@example.com");
    }

    #[tokio::test]
    async fn logout_returns_message() {
        let app = test_app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let msg: MessageResponse =
            serde_json::from_slice(&to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(msg.message, "Successfully logged out");
    }
}
