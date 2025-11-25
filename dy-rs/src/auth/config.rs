//! Authentication configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Secret key for signing JWT tokens (use a strong random string in production!)
    pub jwt_secret: String,

    /// Access token expiration time in seconds (default: 15 minutes)
    pub access_token_expiry_secs: u64,

    /// Refresh token expiration time in seconds (default: 7 days)
    pub refresh_token_expiry_secs: u64,

    /// Issuer claim for JWT tokens
    pub issuer: String,

    /// Audience claim for JWT tokens
    pub audience: String,

    /// Argon2 memory cost (default: 65536 KB = 64 MB)
    pub argon2_memory_cost: u32,

    /// Argon2 time cost (default: 3 iterations)
    pub argon2_time_cost: u32,

    /// Argon2 parallelism (default: 4 threads)
    pub argon2_parallelism: u32,
}

impl AuthConfig {
    /// Create a new AuthConfig with custom JWT secret
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            ..Default::default()
        }
    }

    /// Set access token expiry duration
    pub fn access_token_expiry(mut self, duration: Duration) -> Self {
        self.access_token_expiry_secs = duration.as_secs();
        self
    }

    /// Set refresh token expiry duration
    pub fn refresh_token_expiry(mut self, duration: Duration) -> Self {
        self.refresh_token_expiry_secs = duration.as_secs();
        self
    }

    /// Set the issuer claim
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Set the audience claim
    pub fn audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = audience.into();
        self
    }

    /// Load auth config from environment variables
    ///
    /// Environment variables:
    /// - `AUTH_JWT_SECRET` (required in production)
    /// - `AUTH_ACCESS_TOKEN_EXPIRY_SECS`
    /// - `AUTH_REFRESH_TOKEN_EXPIRY_SECS`
    /// - `AUTH_ISSUER`
    /// - `AUTH_AUDIENCE`
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(secret) = std::env::var("AUTH_JWT_SECRET") {
            config.jwt_secret = secret;
        }

        if let Ok(expiry) = std::env::var("AUTH_ACCESS_TOKEN_EXPIRY_SECS") {
            if let Ok(secs) = expiry.parse() {
                config.access_token_expiry_secs = secs;
            }
        }

        if let Ok(expiry) = std::env::var("AUTH_REFRESH_TOKEN_EXPIRY_SECS") {
            if let Ok(secs) = expiry.parse() {
                config.refresh_token_expiry_secs = secs;
            }
        }

        if let Ok(issuer) = std::env::var("AUTH_ISSUER") {
            config.issuer = issuer;
        }

        if let Ok(audience) = std::env::var("AUTH_AUDIENCE") {
            config.audience = audience;
        }

        config
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            // WARNING: Change this in production!
            jwt_secret: "dy-rs-dev-secret-change-me-in-production".to_string(),
            access_token_expiry_secs: 15 * 60, // 15 minutes
            refresh_token_expiry_secs: 7 * 24 * 60 * 60, // 7 days
            issuer: "dy-rs".to_string(),
            audience: "dy-rs-api".to_string(),
            argon2_memory_cost: 65536, // 64 MB
            argon2_time_cost: 3,
            argon2_parallelism: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthConfig;
    use std::env;
    use std::time::Duration;

    #[test]
    fn builder_helpers_set_expected_values() {
        let cfg = AuthConfig::new("secret")
            .access_token_expiry(Duration::from_secs(10))
            .refresh_token_expiry(Duration::from_secs(20))
            .issuer("issuer")
            .audience("aud");

        assert_eq!(cfg.jwt_secret, "secret");
        assert_eq!(cfg.access_token_expiry_secs, 10);
        assert_eq!(cfg.refresh_token_expiry_secs, 20);
        assert_eq!(cfg.issuer, "issuer");
        assert_eq!(cfg.audience, "aud");
    }

    #[test]
    fn env_overrides_apply_when_present() {
        unsafe {
            env::set_var("AUTH_JWT_SECRET", "env-secret");
            env::set_var("AUTH_ACCESS_TOKEN_EXPIRY_SECS", "111");
            env::set_var("AUTH_REFRESH_TOKEN_EXPIRY_SECS", "222");
            env::set_var("AUTH_ISSUER", "env-iss");
            env::set_var("AUTH_AUDIENCE", "env-aud");
        }

        let cfg = AuthConfig::from_env();
        assert_eq!(cfg.jwt_secret, "env-secret");
        assert_eq!(cfg.access_token_expiry_secs, 111);
        assert_eq!(cfg.refresh_token_expiry_secs, 222);
        assert_eq!(cfg.issuer, "env-iss");
        assert_eq!(cfg.audience, "env-aud");

        for key in [
            "AUTH_JWT_SECRET",
            "AUTH_ACCESS_TOKEN_EXPIRY_SECS",
            "AUTH_REFRESH_TOKEN_EXPIRY_SECS",
            "AUTH_ISSUER",
            "AUTH_AUDIENCE",
        ] {
            unsafe { env::remove_var(key) };
        }
    }
}
