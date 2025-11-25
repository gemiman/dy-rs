# 更新日志

本文件记录项目的所有重要变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)，
并遵守 [语义化版本](https://semver.org/spec/v2.0.0.html)。

## [0.2.0] - 2025-11-22

### 新增
- 🔐 **认证与授权（阶段 2）**
  - 基于 JWT 的访问与刷新令牌
  - Argon2id 密码哈希（行业标准安全性）
  - 受保护路由的 `AuthUser` 提取器
  - 可选认证的 `OptionalAuthUser`
  - 基于角色的访问控制：`require_role()`、`require_any_role()`、`require_all_roles()`
  - 自定义数据库后端的 `UserStore` trait
  - 开发/测试用的 `InMemoryUserStore`
  - 内置认证路由：`/auth/login`、`/auth/register`、`/auth/refresh`、`/auth/logout`、`/auth/me`
  - 密码强度校验
  - 可配置的令牌过期时间
  - 环境变量配置（如 `AUTH_JWT_SECRET`）
- 新增展示认证的 `auth-api` 示例
- 新增 `AUTH.md` 认证文档

### 变更
- 认证功能默认启用（使用 `default-features = false` 可禁用）
- 启用 auth 功能时，预导入中包含 `AuthUser` 与 `AuthConfig`

## [0.1.4] - 2025-11-19

### 修复
- 修复 README 中的示例链接，指向 GitHub 仓库

## [0.1.3] - 2025-11-19

### 修复
- 将 README.md 加入包清单以便 crates.io 展示

## [0.1.2] - 2025-11-18

### 变更
- **默认端口从 3000 改为 8080**，避免 Windows 权限问题
- 更新所有文档以反映 8080 端口
- 更新 CLI 模板使用 8080 端口

### 修复
- 解决 Windows 上 3000 端口的权限不足错误
- 改进跨平台兼容性

## [0.1.1] - 2025-11-18

### 变更
- **不兼容变更**：通过特性开关可选启用 Swagger UI（默认启用）
- 将 `utoipa-swagger-ui` 从 v7.0 降级到 v6.0 以提升稳定性
- 更新文档，提供 Swagger UI 配置说明

### 修复
- 解决 `utoipa-swagger-ui` v7.0 下载失败导致的安装问题
- 在禁用 Swagger UI 特性时改进错误信息

### 新增
- `swagger-ui` 特性开关（默认启用）
- README 中关于禁用/启用 Swagger UI 的说明
- 当 Swagger UI 被禁用时的提示日志

## [0.1.0] - 2025-11-18

### 新增
- 首个发布！🎉
- 通过 `App::new().auto_configure()` 零配置启动应用
- 使用 `ValidatedJson<T>` 的请求校验
- 统一的 `ApiError` 和 `ApiResult` 错误处理
- 自动生成的 OpenAPI 与 Swagger UI 文档
- 来自 TOML 与环境变量的类型安全配置
- 带请求关联的结构化日志（tracing）
- 具有合理默认值的 CORS
- `/health` 健康检查
- 脚手架 CLI（`dy new`）
- 热重载支持（`dy dev`）
- 含完整 CRUD 的 REST API 示例

### 框架特性
- 基于 Axum 0.7，性能优秀
- 默认异步，使用 Tokio
- 编译期类型安全
- 约定优于配置
- 生产就绪的可观测性
