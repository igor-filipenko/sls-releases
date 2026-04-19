//! Types for the transactions HTTP handler.

use chrono::NaiveDateTime;
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: i64,
    pub created: NaiveDateTime,
}
