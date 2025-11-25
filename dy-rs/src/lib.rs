//! # dy-rs
//!
//! Zero-config, batteries-included web framework for Rust.
//! FastAPI meets Spring Boot, powered by Axum.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dy_rs::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     App::new()
//!         .auto_configure()
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ## With Authentication
//!
//! ```rust,ignore
//! use dy_rs::prelude::*;
//! use dy_rs::auth::{AuthConfig, auth_routes};
//!
//! #[tokio::main]
//! async fn main() {
//!     let auth_config = AuthConfig::from_env();
//!     
//!     App::new()
//!         .auto_configure()
//!         .mount(auth_routes(auth_config))
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod app;
pub mod config;
pub mod error;
pub mod extractors;
pub mod openapi;
pub mod prelude;

#[cfg(feature = "auth")]
pub mod auth;

pub use app::App;
pub use dy_rs_macros::dy_api;
pub use error::{ApiError, ApiResult};
pub use extractors::ValidatedJson;
