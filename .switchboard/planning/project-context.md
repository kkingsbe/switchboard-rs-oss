# Project Context — Discord Gateway Service

> Project Type: brownfield (incremental)
> This file is read by every development agent before implementation.
> Keep it concise. Only include rules that prevent common mistakes.

## Build & Test Commands

- **Build:** `cargo build --features "discord gateway"` (include gateway feature flag)
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Format:** `cargo fmt`

## Technology Stack

- **Language:** Rust 2021 edition
- **Async Runtime:** tokio 1.40 (full features)
- **Discord:** twilight-gateway 0.17
- **WebSocket:** tokio-tungstenite 0.24
- **HTTP:** axum 0.7
- **Error Handling:** thiserror 1.0
- **Logging:** tracing 0.1

## Critical Rules

1. **Error Handling:** Use `thiserror` for error types. Never use `anyhow` in library code. Follow patterns in `src/discord/gateway.rs`.

2. **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages. Never use `unwrap()` outside tests.

3. **Async conventions:** Use tokio for async. Follow patterns in `src/discord/gateway.rs` - `async fn` with `tokio::main`.

4. **Module organization:** New gateway code goes in `src/gateway/`. CLI commands go in `src/cli/commands/gateway.rs`.

5. **Configuration:** Use TOML config files with serde. Follow patterns in `src/config/`.

6. **Testing:** Place unit tests in the same file as the code (module tests). Use descriptive test names: `test_name_should_do_something()`.

7. **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`.

8. **Serialization:** Use `serde` and `serde_json` for JSON. Use `toml` for config files.

## File Organization

- Gateway module: `src/gateway/mod.rs` + submodules
- CLI commands: `src/cli/commands/gateway.rs`
- Tests: Inline in module files (`#[cfg(test)] mod tests`)

## Naming Conventions

- Functions: `snake_case` (e.g., `connect_with_shutdown`)
- Types: `PascalCase` (e.g., `GatewayConfig`, `DiscordEvent`)
- Files: `snake_case.rs` (e.g., `gateway.rs`, `config.rs`)

## Patterns to Follow

- Error handling: See architecture.md §7
- Module structure: See architecture.md §3
- Async patterns: See `src/discord/gateway.rs`

## Anti-Patterns (Do NOT)

- Do NOT use `unwrap()` in production code
- Do NOT add dependencies without updating Cargo.toml
- Do NOT use `println!` or `eprintln!` - use `tracing` instead
- Do NOT mix blocking and async code incorrectly

## Skills Reference

- [Rust Best Practices](./skills/rust-best-practices/SKILL.md)
- [Rust Engineer](./skills/rust-engineer/SKILL.md)
