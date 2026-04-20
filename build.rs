use std::path::Path;

fn main() {
    // Only validate `web/dist` when we're actually embedding it.
    if std::env::var_os("CARGO_FEATURE_EMBEDDED_WEB").is_none() {
        return;
    }

    println!("cargo:rerun-if-changed=web/dist");

    let index = Path::new("web/dist/index.html");
    if !index.exists() {
        panic!(
            "Missing web/dist. Build the frontend first (e.g. `cd web && bun install --frozen-lockfile && bun run build`) \
then rebuild this crate."
        );
    }
}

