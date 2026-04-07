# AGENTS.md

This file defines working conventions for agents and humans while rewriting this service in Rust.

## Goals
- **Feature parity first**: preserve endpoint paths, query params, response formats (HTML vs CSV), ordering rules, and error behaviors.
- **Keep it small**: this is a tiny service; avoid over-architecture.
- **Predictable runs**: local run via `cargo run` with `GITHUB_TOKEN` set.

## Non-goals
- No Docker work (no `Dockerfile`, no `docker-compose` changes) unless explicitly requested later.
- No new external APIs or unrelated endpoints (health/metrics) unless explicitly requested.

## Required parity checks
- **Accept handling**: treat as HTML if the `Accept` header contains substring `html`.
- **`rc` param**: Kotlin `.toBoolean()` semantics (only `"true"` is true).
- **Version ordering**: Release > RC for the same major/minor/patch; RC compares by its number.
- **CSV/HTML formatting**: keep comma+space in CSV and the same `<table rules="all">...` HTML layout.
- **Transaction decoding**: Base62 decode into two numbers, convert epoch seconds using a fixed server offset (startup offset) and return `400` on decode failure.

## Suggested Rust crate layout
- `src/main.rs`: configuration + router + server.
- `src/config.rs`: config types + load logic.
- `src/clients/github/`: GitHub paging client + DTOs + cache.
- `src/domain/`: release/version domain types and ordering.
- `src/routes/`: axum handlers.
- `src/render.rs`: CSV/HTML rendering helpers.
- `tests/`: parity tests. Prefer unit tests over heavy integration tests.

## Dependency guidelines
- Prefer mature, common crates: `axum`, `tokio`, `reqwest`, `serde`, `config`, `regex`, `chrono`, `tracing`.
- Minimize dependencies unless they reduce complexity substantially (e.g., caching, base62).

## Workflow expectations
- Add tests when encoding behavior that is easy to regress (version ordering, tag parsing, output formatting).
- Keep commits scoped (one behavior change per commit) once we start executing.

