//! Authentication and Authorization module for dy-rs
//!
//! Provides JWT-based authentication, password hashing, and route protection.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use dy_rs::prelude::*;
//! use dy_rs::auth::{AuthConfig, AuthUser, login, register};
//!
//! #[tokio::main]
//! async fn main() {
//!     App::new()
//!         .auto_configure()
//!         .with_auth(AuthConfig::default())
//!         .mount(auth_routes())
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod config;
pub mod extractors;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod models;
pub mod password;

pub use config::AuthConfig;
pub use extractors::AuthUser;
pub use handlers::{
    AuthAppState, CreateUserData, InMemoryUserStore, StoredUser, UserStore, auth_routes,
    auth_routes_with_store, login, logout, refresh_token, register,
};
pub use jwt::{Claims, TokenPair, create_token_pair, verify_token};
pub use middleware::RequireAuth;
pub use models::{AuthResponse, LoginRequest, RegisterRequest, TokenRefreshRequest};
pub use password::{hash_password, verify_password};
