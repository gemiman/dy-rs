# 🔐 认证与授权

dy-rs 内置完整的基于 JWT 的认证系统。

## 快速开始

```rust
use dy_rs::prelude::*;
use dy_rs::auth::{AuthConfig, auth_routes};

#[tokio::main]
async fn main() {
    let auth_config = AuthConfig::from_env();
    
    App::new()
        .auto_configure()
        .mount(auth_routes(auth_config))
        .run()
        .await
        .unwrap();
}
```

自动提供以下端点：
- `POST /auth/register` - 注册新用户
- `POST /auth/login` - 登录并获取 JWT
- `POST /auth/refresh` - 刷新访问令牌
- `POST /auth/logout` - 退出登录
- `GET /auth/me` - 获取当前用户信息（受保护）

## 配置

### 环境变量

```bash
# 生产必填——使用强随机字符串
AUTH_JWT_SECRET=your-super-secret-key-min-32-chars

# 可选
AUTH_ACCESS_TOKEN_EXPIRY_SECS=900      # 15 分钟（默认）
AUTH_REFRESH_TOKEN_EXPIRY_SECS=604800  # 7 天（默认）
AUTH_ISSUER=your-app-name
AUTH_AUDIENCE=your-api
```

### 编程方式配置

```rust
use dy_rs::auth::AuthConfig;
use std::time::Duration;

let config = AuthConfig::new("your-secret-key")
    .access_token_expiry(Duration::from_secs(30 * 60))  // 30 分钟
    .refresh_token_expiry(Duration::from_secs(14 * 24 * 60 * 60))  // 14 天
    .issuer("my-app")
    .audience("my-api");
```

## 保护路由

使用 `AuthUser` 提取器保护路由：

```rust
use dy_rs::prelude::*;
use dy_rs::auth::AuthUser;

// 需要认证
async fn protected_route(user: AuthUser) -> impl IntoResponse {
    format!("Hello, {}!", user.email)
}

// 仅管理员
async fn admin_only(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    user.require_role("admin")?;
    Ok("Welcome, admin!")
}

// 检查多个角色
async fn editor_or_admin(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    user.require_any_role(&["editor", "admin"])?;
    Ok("You have edit access!")
}
```

## 可选认证

使用 `OptionalAuthUser` 支持“有/无认证均可”的路由：

```rust
use dy_rs::auth::OptionalAuthUser;

async fn maybe_personalized(user: OptionalAuthUser) -> impl IntoResponse {
    match user.0 {
        Some(user) => format!("Hello, {}!", user.email),
        None => "Hello, anonymous!".to_string(),
    }
}
```

## 自定义用户存储

默认使用内存存储（仅限开发）。生产环境请为数据库实现 `UserStore`：

```rust
use dy_rs::auth::{UserStore, StoredUser, CreateUserData};
use sqlx::PgPool;

struct PostgresUserStore {
    pool: PgPool,
}

#[async_trait]
impl UserStore for PostgresUserStore {
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
        let row = sqlx::query!(
            "SELECT id, email, name, password_hash, roles FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| StoredUser {
            id: r.id.to_string(),
            email: r.email,
            name: r.name,
            password_hash: r.password_hash,
            roles: r.roles,
        }))
    }
    
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("Invalid ID".into()))?;
        
        let row = sqlx::query!(
            "SELECT id, email, name, password_hash, roles FROM users WHERE id = $1",
            uuid
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| StoredUser {
            id: r.id.to_string(),
            email: r.email,
            name: r.name,
            password_hash: r.password_hash,
            roles: r.roles,
        }))
    }
    
    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError> {
        let id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, name, password_hash, roles)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            user.email,
            user.name,
            user.password_hash,
            &["user".to_string()][..],
        )
        .execute(&self.pool)
        .await?;
        
        Ok(StoredUser {
            id: id.to_string(),
            email: user.email,
            name: user.name,
            password_hash: user.password_hash,
            roles: vec!["user".to_string()],
        })
    }
    
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("Invalid ID".into()))?;
        
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            password_hash,
            uuid
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn email_exists(&self, email: &str) -> Result<bool, ApiError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            email
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(exists.unwrap_or(false))
    }
}
```

然后这样使用：

```rust
use dy_rs::auth::auth_routes_with_store;

let pool = PgPool::connect(&database_url).await?;
let user_store = PostgresUserStore { pool };

App::new()
    .auto_configure()
    .mount(auth_routes_with_store(auth_config, user_store))
    .run()
    .await?;
```

## 密码哈希

dy-rs 使用 Argon2id（推荐算法）进行密码哈希：

```rust
use dy_rs::auth::{hash_password, verify_password, AuthConfig};

let config = AuthConfig::default();

// 生成哈希
let hash = hash_password("my-password", &config)?;

// 验证密码
let is_valid = verify_password("my-password", &hash)?;
```

### 密码校验

```rust
use dy_rs::auth::password::{validate_password_strength, PasswordValidator};

// 默认校验（8+ 字符，含大小写与数字）
validate_password_strength("SecurePass1")?;

// 自定义校验
let validator = PasswordValidator::new()
    .min_length(12)
    .require_special(true);

validator.validate("SecurePass1!")?;
```

## API 参考

### 登录

```bash
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123"
}
```

响应：
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "name": "John Doe",
    "roles": ["user"]
  }
}
```

### 注册

```bash
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123",
  "name": "John Doe"
}
```

### 刷新令牌

```bash
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}
```

### 访问受保护路由

```bash
GET /protected
Authorization: Bearer <access_token>
```

## 安全最佳实践

1. **使用强 JWT 密钥**——至少 32 位随机字符串
2. **安全存放密钥**——用环境变量而非硬编码
3. **生产必须启用 HTTPS**
4. **访问令牌短时效**——15 分钟或更短
5. **实现令牌刷新**——避免频繁重新登录
6. **实现限流**——防止暴力破解
7. **记录安全事件**——失败登录、令牌刷新等

## 禁用认证

如果不需要认证：

```toml
[dependencies]
dy-rs = { version = "0.1", default-features = false, features = ["swagger-ui"] }
```
