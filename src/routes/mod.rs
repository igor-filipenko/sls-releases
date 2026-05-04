use axum::http::StatusCode;

use crate::persistence::PersistenceError;

pub mod dto;
pub mod modules;
pub mod releases;
mod render;
pub mod transactions;
pub mod web;

/// Logs persistence failures and returns an HTTP status for route handlers.
pub(crate) fn map_store_error(route: &'static str, err: PersistenceError) -> StatusCode {
    let status = match &err {
        PersistenceError::InvalidVersionKind(_) => StatusCode::INTERNAL_SERVER_ERROR,
        PersistenceError::Sql(e) => match e {
            sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::WorkerCrashed => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::BAD_GATEWAY,
        },
    };
    tracing::error!(
        route,
        error = %err,
        status = status.as_u16(),
        "store operation failed"
    );
    status
}
