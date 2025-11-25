# 为 dy-rs 贡献

感谢你愿意贡献 dy-rs！这是一个早期项目，欢迎各种形式的贡献。

## 如何贡献

### 报告缺陷

请在 GitHub 提交 issue，并提供：
- 清晰的缺陷描述
- 复现步骤
- 预期与实际行为
- 你的环境信息（OS、Rust 版本等）

### 提议功能

提交 issue，并说明：
- 功能描述与动机
- 主要使用场景
- 可能的实现思路

### 提交 Pull Request

1. Fork 仓库
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 完成改动
4. 如适用，添加测试
5. 运行 `cargo test` 与 `cargo clippy`
6. 提交变更（`git commit -m 'Add amazing feature'`）
7. 推送分支（`git push origin feature/amazing-feature`）
8. 发起 PR

## 开发环境

```bash
git clone https://github.com/ashishjsharda/dy-rs
cd dy-rs
cargo build
cargo test

# 运行示例
cd examples/rest-api
cargo run
```

## 代码风格

- 遵循 Rust 惯例与习惯用法
- 提交前运行 `cargo fmt`
- 运行 `cargo clippy` 并修复警告
- 为公开 API 添加文档注释

## 测试

- 为新功能编写测试
- 提交前确保所有测试通过
- 需要时添加集成测试

## 问题交流

可提交 issue 或联系 [@ashishjsharda](https://github.com/ashishjsharda)

## 行为准则

保持尊重、包容、建设性；一起把项目做好。
