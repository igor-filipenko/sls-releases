use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;

#[cfg(not(feature = "embedded-web"))]
pub async fn fallback(_uri: Uri) -> impl IntoResponse {
    StatusCode::NOT_FOUND
}

#[cfg(feature = "embedded-web")]
mod embedded {
    use axum::body::Body;
    use axum::http::{header, HeaderValue, Response, StatusCode, Uri};
    use axum::response::IntoResponse;
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "web/dist"]
    struct WebDist;

    pub async fn fallback(uri: Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');

        if path.starts_with("sls/") {
            return StatusCode::NOT_FOUND.into_response();
        }

        if path.is_empty() {
            return serve_path("index.html");
        }

        if let Some(resp) = serve_if_exists(path) {
            return resp;
        }

        // SPA routing: only fall back on "route-like" paths, not on missing asset-like paths.
        if !path.contains('.') {
            return serve_path("index.html");
        }

        StatusCode::NOT_FOUND.into_response()
    }

    fn serve_if_exists(path: &str) -> Option<Response<Body>> {
        WebDist::get(path).map(|f| response_for_file(path, f.data.into_owned()))
    }

    fn serve_path(path: &str) -> Response<Body> {
        match WebDist::get(path) {
            Some(f) => response_for_file(path, f.data.into_owned()),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    fn response_for_file(path: &str, bytes: Vec<u8>) -> Response<Body> {
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        let mut resp = Response::new(Body::from(bytes));
        *resp.status_mut() = StatusCode::OK;

        let headers = resp.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref())
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
        );

        // Conservative caching: keep `index.html` fresh; allow long caching for Vite fingerprinted assets.
        if path == "index.html" {
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        } else if path.starts_with("assets/") {
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        }

        resp
    }
}

#[cfg(feature = "embedded-web")]
pub use embedded::fallback;

