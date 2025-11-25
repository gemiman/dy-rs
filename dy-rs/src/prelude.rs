//! Convenient re-exports for common types
//!
//! Use `use dy_rs::prelude::*;` to get everything you need

pub use crate::{
    app::App,
    error::{ApiError, ApiResult},
    extractors::ValidatedJson,
};

// Re-export commonly used types from dependencies
pub use axum::{
    Router,
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{delete, get, patch, post, put},
};

pub use serde::{Deserialize, Serialize};
pub use validator::Validate;

pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

pub use utoipa::ToSchema;

// Auth re-exports (when auth feature is enabled)
#[cfg(feature = "auth")]
pub use crate::auth::{AuthConfig, AuthUser};
