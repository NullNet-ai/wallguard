use chrono::{DateTime, Utc};

pub fn timestamp() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn timestamp_to_iso_string(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(timestamp)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| format!("Timestamp {timestamp} ms"))
}
