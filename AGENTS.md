# Repository Guidelines

[中文文档](AGENTS-zh.md)

## Project Structure & Module Organization
- Workspace root defines shared tooling and dependencies (`Cargo.toml`, `Cargo.lock`).
- `dy-rs/`: core framework crate (Axum-based server, config, validation, logging).
- `dy-rs-macros/`: procedural macros used by the core crate.
- `dy-rs-cli/`: CLI for scaffolding and dev workflows (e.g., `dy new`, `dy dev`).
- `examples/rest-api` and `examples/auth-api`: runnable samples; start here when testing changes.

## Build, Test, and Development Commands
```bash
cargo build                # Build all workspace crates
cargo test                 # Run unit/integration tests
cargo fmt                  # Format the workspace
cargo clippy --all-targets --all-features  # Lint with warnings as errors
cd examples/rest-api && cargo run         # Launch example API locally
```
Use `APP__SERVER__PORT=8080 cargo run` in an example to override ports.

## Coding Style & Naming Conventions
- Follow Rust 2024 idioms; keep public APIs documented with `///`.
- Prefer snake_case for files/modules, CamelCase for types, and clear verb-based function names.
- Run `cargo fmt` and `cargo clippy` before pushing; fix all warnings.
- Keep modules small and cohesive (routes, services, config separated by concern).

## Testing Guidelines
- Add unit tests near implementation files and integration tests under `tests/` when cross-crate behavior is involved.
- Match test names to behavior (`test_creates_user_on_valid_payload`).
- For examples, favor request-level tests over mocks where possible.
- Aim for coverage of new branches and error paths; avoid skipping `clippy`/`fmt` checks.

## Commit & Pull Request Guidelines
- Commit messages follow conventional style visible in history (`feat: …`, `chore: …`, `docs: …`).
- Keep commits scoped and reviewable; include tests and formatting changes with the code that needs them.
- Pull requests should: describe the change and rationale, link related issues, note breaking changes, and include manual/automated test results. Add screenshots or cURL snippets when touching routes or docs.

## Configuration & Security Notes
- Examples load layered config from `config/*.toml` plus `APP__*` environment vars; avoid committing secrets and prefer `.env` for local overrides.
- When adding new services, expose configuration through typed structs and validate defaults to keep zero-config behavior intact.
