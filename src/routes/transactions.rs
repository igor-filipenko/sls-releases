use crate::routes::dto::transactions::Transaction;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, FixedOffset, Utc};

#[derive(Clone)]
pub struct TransactionsState {
    pub zone_offset: FixedOffset,
}

pub fn router(state: TransactionsState) -> Router {
    Router::new()
        .route("/sls/transactions/{id}", get(get_transaction))
        .with_state(state)
}

async fn get_transaction(
    State(state): State<TransactionsState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match decode_array(&id).and_then(|parts| parts.first().copied().zip(parts.get(1).copied())) {
        Some((internal_id, seconds)) => {
            let dt_utc: DateTime<Utc> = match DateTime::<Utc>::from_timestamp(seconds, 0) {
                Some(dt) => dt,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        format!("Invalid transaction ID: '{id}'"),
                    )
                        .into_response();
                }
            };
            let created = dt_utc.with_timezone(&state.zone_offset).naive_local();
            (
                StatusCode::OK,
                Json(Transaction {
                    id: internal_id,
                    created,
                }),
            )
                .into_response()
        }
        None => {
            // Kotlin prints stack trace; we just return the same HTTP shape.
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid transaction ID: '{id}'"),
            )
                .into_response()
        }
    }
}

fn decode_array(input: &str) -> Option<Vec<i64>> {
    if !input.len().is_multiple_of(11) {
        return None;
    }
    let mut out = Vec::with_capacity(input.len() / 11);
    let chars: Vec<char> = input.chars().collect();
    for chunk in chars.chunks(11) {
        out.push(decode_long_11(chunk)?);
    }
    Some(out)
}

fn decode_long_11(chunk: &[char]) -> Option<i64> {
    if chunk.len() != 11 {
        return None;
    }

    let mut negative = false;
    let mut digit = digit_index(chunk[0])?;
    if digit >= 31 {
        digit -= 31;
        negative = true;
    }
    let mut value: i64 = digit as i64;
    for ch in &chunk[1..] {
        let d = digit_index(*ch)? as i64;
        value = value.checked_mul(62)?.checked_add(d)?;
    }
    if negative {
        value = -value;
    }
    Some(value)
}

fn digit_index(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some((ch as u32) - ('0' as u32)),
        'A'..='Z' => Some(10 + (ch as u32) - ('A' as u32)),
        'a'..='z' => Some(36 + (ch as u32) - ('a' as u32)),
        _ => None,
    }
}
