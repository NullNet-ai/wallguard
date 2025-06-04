use chrono::Utc;

pub fn timestamp() -> i64 {
    Utc::now().timestamp_millis()
}
