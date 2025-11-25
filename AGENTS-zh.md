# 仓库指南

## 项目结构与模块组织
- 工作区根目录存放共享工具与依赖（`Cargo.toml`、`Cargo.lock`）。
- `dy-rs/`：核心框架 crate（基于 Axum 的服务器、配置、校验、日志）。
- `dy-rs-macros/`：核心 crate 使用的过程宏。
- `dy-rs-cli/`：脚手架与开发流程 CLI（如 `dy new`、`dy dev`）。
- `examples/rest-api` 与 `examples/auth-api`：可运行示例，测试变更的首选入口。

## 构建、测试与开发命令
```bash
cargo build                # 构建全部 workspace crate
cargo test                 # 运行单元/集成测试
cargo fmt                  # 格式化整个工作区
cargo clippy --all-targets --all-features  # 以警告即错误方式执行 lint
cd examples/rest-api && cargo run         # 本地启动示例 API
```
在示例中使用 `APP__SERVER__PORT=8080 cargo run` 来覆盖端口。

## 代码风格与命名约定
- 遵循 Rust 2024 习惯；为公开 API 添加 `///` 文档注释。
- 文件/模块用 snake_case，类型用 CamelCase，函数名称直观且以动词开头。
- 推送前运行 `cargo fmt` 与 `cargo clippy`，修复全部警告。
- 保持模块内聚：按路由、服务、配置等关注点分层。

## 测试指南
- 在实现附近添加单元测试，跨 crate 行为放在 `tests/` 做集成测试。
- 测试命名匹配行为（如 `test_creates_user_on_valid_payload`）。
- 示例中优先做请求级测试，少用过度 mock。
- 覆盖新增分支与错误路径；不要跳过 `clippy`/`fmt` 检查。

## 提交与 PR 指南
- 提交信息沿用历史的惯例风格（`feat: …`、`chore: …`、`docs: …`）。
- 保持提交粒度可审阅；代码、测试与格式化放在同一提交内。
- PR 应描述变更与动机、关联 issue、注明破坏性改动，并附上手动/自动测试结果。若涉及路由或文档，附截图或 cURL 片段。

## 配置与安全提示
- 示例通过 `config/*.toml` 与 `APP__*` 环境变量加载分层配置；避免提交密钥，使用 `.env` 做本地覆盖。
- 新增服务时以类型化结构暴露配置并校验默认值，保持零配置体验。
