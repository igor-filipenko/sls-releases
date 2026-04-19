use axum::response::Response;
use http_body_util::BodyExt;

pub async fn body_string(resp: Response) -> String {
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("body collect")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf-8")
}

pub fn csv_non_empty_line_count(s: &str) -> usize {
    s.lines().filter(|l| !l.is_empty()).count()
}

#[path = "releases.rs"]
mod releases;
#[path = "transactions.rs"]
mod transactions;
