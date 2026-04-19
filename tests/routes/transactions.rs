use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{FixedOffset, TimeZone, Utc};
use tower::ServiceExt;

use sls_releases::routes;
use sls_releases::routes::transactions::TransactionsState;

use super::body_string;

#[tokio::test]
async fn transactions_route_valid_id_returns_exact_json() {
    fn encode_long(value: i64) -> String {
        const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut buf = [b'0'; 11];
        let mut v = value;
        if v < 0 {
            for i in (1..=10).rev() {
                let digit = (-(v % 62)) as usize;
                buf[i] = DIGITS[digit];
                v /= 62;
            }
            let first = (-(v - 31)) as usize;
            buf[0] = DIGITS[first];
        } else {
            for i in (1..=10).rev() {
                let digit = (v % 62) as usize;
                buf[i] = DIGITS[digit];
                v /= 62;
            }
            buf[0] = DIGITS[v as usize];
        }
        String::from_utf8(buf.to_vec()).unwrap()
    }

    let internal_id = 123456789i64;
    let seconds = 1710000000i64;
    let id = format!("{}{}", encode_long(internal_id), encode_long(seconds));

    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let app = routes::transactions::router(TransactionsState {
        zone_offset: offset,
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/transactions/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_string(resp).await;
    let dt = Utc.timestamp_opt(seconds, 0).unwrap();
    let created = dt
        .with_timezone(&FixedOffset::east_opt(3 * 3600).unwrap())
        .naive_local();

    let expected = format!(
        "{{\"id\":{internal_id},\"created\":\"{}\"}}",
        created.format("%Y-%m-%dT%H:%M:%S")
    );
    assert_eq!(body, expected);
}

#[tokio::test]
async fn transactions_route_invalid_id_returns_400_and_message() {
    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let app = routes::transactions::router(TransactionsState {
        zone_offset: offset,
    });

    let bad = "not-valid";
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/transactions/{bad}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_string(resp).await;
    assert_eq!(body, format!("Invalid transaction ID: '{bad}'"));
}
