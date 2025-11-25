# 🚀 dy-rs

> **零配置、开箱即用的 Rust Web 框架**  
> FastAPI + Spring Boot 的体验，基于 Axum 驱动

[![Crates.io](https://img.shields.io/crates/v/dy-rs.svg)](https://crates.io/crates/dy-rs)
[![Documentation](https://docs.rs/dy-rs/badge.svg)](https://docs.rs/dy-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 为什么选择 dy-rs？

构建 Rust Web API 不该需要接十几个 crate、写上百行样板。**dy-rs** 带来 FastAPI 与 Spring Boot 的生产力，同时保留 Rust 的性能与类型安全。

### ⚡ 特性

- 🎯 **零配置** - 数据库、迁移、CORS、日志开箱即用
- 🔒 **类型安全** - 路由、校验、序列化均有编译期保障
- 📚 **自动生成文档** - 基于代码生成 Swagger UI 与 OpenAPI
- ✅ **内置校验** - 友好的请求校验错误提示
- 🔥 **热重载** - `dy dev` 提供快速迭代
- 🎨 **约定式结构** - 约定优于配置
- 🚀 **生产就绪** - 结构化日志、错误处理、健康检查

## 快速开始

### 安装

```bash
cargo install dy-rs-cli
```

**提示：** dy-rs 默认包含 Swagger UI。如果安装遇到问题，可不带默认特性安装：

```bash
cargo add dy-rs --no-default-features
```

稍后启用 Swagger UI：

```bash
cargo add dy-rs --features swagger-ui
```

### 创建你的第一个 API

```bash
# 创建新项目
dy new myapi

# 运行
cd myapi
cargo run
```

你的 API 现已运行：
- 🌐 **http://localhost:8080** - API 端点
- 📚 **http://localhost:8080/docs** - Swagger UI
- 💚 **http://localhost:8080/health** - 健康检查

### 第一个端点

```rust
use dy_rs::prelude::*;

#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 2))]
    name: String,
    
    #[validate(email)]
    email: String,
}

async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUser>
) -> ApiResult<User> {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
    };
    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    App::new()
        .auto_configure()
        .route("/users", post(create_user))
        .run()
        .await
        .unwrap();
}
```

你将得到：
- ✅ 自动请求校验
- ✅ 类型安全的 JSON 序列化
- ✅ 结构化错误响应
- ✅ OpenAPI 文档
- ✅ 请求追踪与日志

## 对比

| 功能 | FastAPI | Spring Boot | **dy-rs** |
|---------|---------|-------------|--------------|
| 类型安全 | ❌ 运行期 | ⚠️ 运行期 | ✅ 编译期 |
| 自动 OpenAPI | ✅ | ✅ | ✅ |
| 热重载 | ✅ | ✅ | ✅ |
| 零配置 | ✅ | ✅ | ✅ |
| 性能 | ⚠️ 良好 | ⚠️ 良好 | ✅ 极快 |
| 内存安全 | ❌ | ❌ | ✅ 保证 |
| 默认异步 | ⚠️ 部分 | ❌ | ✅ |
| 学习曲线 | 简单 | 中等 | 简单 |

## 包含内容

### 🎁 开箱即用

- **配置管理** - TOML + 环境变量
- **数据库集成** - PostgreSQL 连接池（SQLx）
- **请求校验** - 派生式校验，错误友好
- **错误处理** - 统一错误处理与 HTTP 状态码
- **CORS** - 合理默认，可配置
- **日志与追踪** - 结构化日志，带请求关联
- **健康检查** - `/health` 端点
- **OpenAPI/Swagger** - 默认开启的自动文档（`swagger-ui` 特性）

### 📚 Swagger UI 配置

**默认开启** - 默认特性包含 Swagger UI：

```toml
[dependencies]
dy-rs = "0.1"  # 包含 Swagger UI
```

**如需禁用**（减小二进制、加快编译）：

```toml
[dependencies]
dy-rs = { version = "0.1", default-features = false }
```

**重新启用**：

```toml
[dependencies]
dy-rs = { version = "0.1", features = ["swagger-ui"] }
```

### 📦 CLI 工具

```bash
# 使用模板创建新项目
dy new myapi --template rest-api

# 热重载运行
dy dev

# 即将推出：
# dy generate resource User
# dy db migrate
```

## 配置

配置按优先级加载：

1. `config/default.toml` - 基础配置
2. `config/local.toml` - 本地覆盖（已 gitignore）
3. 环境变量 - 前缀 `APP__`

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://localhost/mydb"
max_connections = 10
```

环境变量覆盖：
```bash
APP__SERVER__PORT=8080 cargo run
```

## 示例

查看 [examples](https://github.com/gemiman/dy-rs/tree/main/examples) 目录：

- ✅ **REST API** - 完整 CRUD 与校验
- 🔜 **GraphQL API** - 即将到来
- 🔜 **gRPC Service** - 即将到来
- 🔜 **WebSocket Chat** - 即将到来

## 路线图

### 阶段 1（当前）
- [x] 自动配置的核心框架
- [x] 请求校验
- [x] OpenAPI 生成
- [x] 项目脚手架 CLI
- [x] 热重载

### 阶段 2（下一步）
- [ ] 认证与授权（JWT、会话）
- [ ] 数据库迁移管理
- [ ] 测试工具
- [ ] 更多模板（GraphQL、gRPC）

### 阶段 3（未来）
- [ ] 后台任务
- [ ] 多租户支持
- [ ] 特性开关
- [ ] 管理面板生成

## 贡献

欢迎贡献！项目早期，有大量可施展空间。

### 开发环境

```bash
git clone https://github.com/gemiman/dy-rs
cd dy-rs
cargo build
cargo test

# 运行示例
cd examples/rest-api
cargo run
```

## 理念

**dy-rs** 基于以下原则：

1. **约定优于配置** - 合理默认，减少样板
2. **类型安全优先** - 依靠 Rust 类型系统捕获错误
3. **开发者体验** - 让常见场景简单，复杂场景可行
4. **生产就绪** - 默认包含可观测性、错误处理与最佳实践
5. **可组合** - 构建于 Axum，可按需使用 Axum 模式

## 为什么不直接用 Axum？

**Axum** 很棒——它是 dy-rs 的基石！但 Axum 有意保持最小化与非侵入，你需要自行接好：

- 配置加载
- 数据库连接
- 校验
- 错误处理模式
- OpenAPI 生成
- 日志设置
- CORS
- 项目结构

**dy-rs** 将这些开箱即用，同时保留 Axum 的全部能力。

## 许可

可任选其一：

- Apache License 2.0（[LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0）
- MIT license（[LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT）

## 致谢

由 [Gemiman](https://github.com/gemiman) 构建

站在巨人肩膀上：
- [Axum](https://github.com/tokio-rs/axum) - 卓越的 Web 框架
- [FastAPI](https://fastapi.tiangolo.com/) - DX 灵感
- [Spring Boot](https://spring.io/projects/spring-boot) - 约定式灵感

---

**觉得有用请点 Star ⭐！**

[报告缺陷](https://github.com/gemiman/dy-rs/issues) · [请求功能](https://github.com/gemiman/dy-rs/issues) · [文档](https://docs.rs/dy-rs)
