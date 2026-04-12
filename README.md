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

From the repository root, run the API (see above), then in another terminal:

```shell
cd web
bun install
bun dev
```

Open the URL printed by Vite (default `http://127.0.0.1:5173`). The dev server proxies `/sls` to `http://127.0.0.1:8080` so the browser can load release CSV without CORS issues. To point at another backend, set `VITE_API_PROXY` when starting Vite (see `web/vite.config.ts`).

Production build: `cd web && bun run build` outputs static files under `web/dist`; serve them on the same origin as the API or enable CORS on the Rust server.