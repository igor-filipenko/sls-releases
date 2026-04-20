# sls-releases

Service to collect last releases for multiproject repo.

Setup:

```shell
rustc --version
cargo --version

# 1) Fill token in ./sls.toml:
#    github.token = "your_token_here"
#
# 2) run
cargo run
```

then check

```shell
curl 'http://0.0.0.0:8080/sls/releases?rc=true'
```

HTML response:

```shell
curl -H 'Accept: text/html' 'http://0.0.0.0:8080/sls/releases?rc=true'
```

## Web UI (Vite + React + shadcn)

### Local dev

From the repository root, run the API (see above), then in another terminal:

```shell
cd web
bun install
bun dev
```

Open the URL printed by Vite (default `http://127.0.0.1:5173`). The dev server proxies `/sls` to `http://127.0.0.1:8080` so the browser can load release CSV without CORS issues. To point at another backend, set `VITE_API_PROXY` when starting Vite (see `web/vite.config.ts`).

### Production (single binary, embedded UI)

The Rust server can embed the built static files from `web/dist` into the binary (feature `embedded-web`) and serve the SPA at `/` (with an `index.html` fallback for client-side routes). API endpoints remain under `/sls/...`.

Release build order:

```shell
cd web
# installs JS deps; `--frozen-lockfile` makes it fail if `bun.lock` is out of date
bun install --frozen-lockfile
bun run build

cd ..
cargo build --release --features embedded-web
```

Notes:

- `cargo build --features embedded-web` expects `web/dist/index.html` to exist (built by Vite).
- Default builds (`cargo run`, `cargo test`) do not require `web/dist` and do not serve the SPA at `/`.
