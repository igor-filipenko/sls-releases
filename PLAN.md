# Rust rewrite plan (feature parity)

This project will be rewritten from Kotlin/Ktor to Rust with **full feature parity**: same endpoints, query params, response formats (HTML vs CSV), version ordering rules, and configuration inputs.

## Current behavior to preserve (feature parity)
- **HTTP endpoints** (from `[src/main/kotlin/ru/crystals/sls/releases/routes/ReleasesRoute.kt](/home/igor/src/my/sls-releases/src/main/kotlin/ru/crystals/sls/releases/routes/ReleasesRoute.kt)` and `[TransactionsRoute.kt](/home/igor/src/my/sls-releases/src/main/kotlin/ru/crystals/sls/releases/routes/TransactionsRoute.kt)`)
  - `GET /sls/releases?rc=true|false`
    - Fetches GitHub releases, filters by version type unless `rc=true`.
    - Groups by module name and picks the max version per module.
    - Responds **HTML table** if `Accept` contains `html`, else **plain text CSV**.
  - `GET /sls/releases/{module}?rc=true|false`
    - Same filtering, then returns **all versions** for that module sorted by version desc.
    - HTML table if `Accept` contains `html`, else CSV.
  - `GET /sls/transactions/{id}`
    - Base62-decodes into `[internalId, seconds]`, converts `seconds` to a `LocalDateTime` using the server’s current offset.
    - Responds JSON on success; `400` on decode errors.

- **GitHub integration** (from `[GitHubClient.kt](/home/igor/src/my/sls-releases/src/main/kotlin/ru/crystals/sls/releases/client/github/GitHubClient.kt)` and `[Converter.kt](/home/igor/src/my/sls-releases/src/main/kotlin/ru/crystals/sls/releases/client/github/Converter.kt)`)
  - Calls GitHub Releases API for `crystalservice/SET10-Loyalty` with paging: `per_page=100&page=N` until an empty page.
  - Uses `Authorization: Bearer <token>`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`.
  - Parses tags with regex `^(.*)-v(\d+).(\d+).(\d+)(-RC\d+)?$`.
  - Filters only modules present in `sls.modules` map (loaded from config).
  - Converts `created_at` to a formatted string `MMM d, yyyy 'at' h:mm a` in system timezone; on parse failure returns original string.
  - Caches each GitHub “page” with TTL derived from response `Cache-Control: max-age=...` (default 60s).

- **Config contract**
  - Port `8080` (from `[application.yaml](/home/igor/src/my/sls-releases/src/main/resources/application.yaml)`).
  - GitHub token comes from env `GITHUB_TOKEN`.
  - Module localization map comes from config `sls.modules`.

## Recommended Rust stack
- **Web**: `axum` + `tokio`.
- **HTTP client**: `reqwest` (JSON via `serde`).
- **Config**: `config` crate to load YAML + env overrides (`GITHUB_TOKEN`, optional config path).
- **Caching**: `moka` (async cache) or `dashmap` + per-entry `expires_at` to mirror current per-page TTL semantics.
- **Parsing**: `regex` for tag parsing; `chrono` for time conversion/format.
- **Serialization**: `serde` / `serde_json`.
- **Base62**: `base62` crate (or equivalent) to match current decoding behavior.
- **Logging**: `tracing` + `tracing-subscriber`.

## Target Rust structure
- `Cargo.toml`
- `src/main.rs` (boot, config load, router build, server bind)
- `src/config.rs` (token, modules map, optional repo owner/name)
- `src/clients/github/mod.rs` (`GitHubClient`, DTO for GitHub release, paging, cache)
- `src/domain/release.rs` (`Version`, `Release`, `ModuleRelease`, ordering)
- `src/routes/releases.rs` and `src/routes/transactions.rs`
- `src/render.rs` (CSV/HTML formatting helpers)
- `tests/` for parity tests (tag parsing, version ordering, outputs)

## Parity-focused implementation notes
- **Content negotiation**: mirror current check `Accept` contains `html` (substring match), not strict `text/html` matching.
- **`rc` query param**: Kotlin uses `.toBoolean()` (only `"true"` -> true). Mirror that exact behavior.
- **HTML output**: replicate the exact table markup and link shapes used by `asHtmlRow` (including `baseUrl/$name?rc=$useCandidate`).
- **CSV output**: keep the same comma+space formatting and trailing newline for the list endpoint.
- **Version ordering**: replicate the Kotlin comparator semantics where final releases sort after candidates of the same major/minor/patch (candidate number uses `Int.MAX_VALUE` for releases).
- **Transaction time**: Kotlin uses `OffsetDateTime.now().offset` at startup; for strict parity, freeze offset at startup.

## Execution steps
1. **Codify contract**: add minimal tests in Rust reproducing Kotlin behavior (tag parsing, version compare, rc filtering, HTML/CSV formatting, base62 decode).
2. **Implement config**: load `GITHUB_TOKEN` + module map from YAML/env.
3. **Implement GitHub client**: paging + headers + per-page cache TTL from `Cache-Control`.
4. **Implement routes**: match endpoint paths, query params, Accept handling, and status codes.
5. **Docs + run**: update `README.md` usage for Rust (e.g. `GITHUB_TOKEN=... cargo run`), and maintain a minimal local dev loop.
6. **Smoke test**: curl the three endpoints with and without `Accept: text/html` and with `rc=true`.

