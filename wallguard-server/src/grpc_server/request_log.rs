use chrono::{DateTime, Utc};
use tonic::{Code, Request, Response, Status};

use crate::proto::wallguard::{Authentication, CommonResponse, StatusResponse};

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

pub(crate) struct ServerLogger {}

impl ServerLogger {
    pub(crate) fn extract_address<T>(request: &Request<T>) -> String {
        request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Unified log response function for different response types.
    pub(crate) fn log_response<T>(
        response: &Result<Response<T>, Status>,
        source: &str,
        destination: &str,
        received_at: DateTime<Utc>,
    ) where
        T: LoggableResponse,
    {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - received_at).num_milliseconds();

        let received_str = Self::format_timestamp(received_at);
        let completed_str = Self::format_timestamp(completed_at);

        match response {
            Ok(success_response) => {
                success_response.get_ref().log_success(
                    &received_str,
                    &completed_str,
                    source,
                    destination,
                    duration_ms,
                );
            }
            Err(status) => {
                Self::log_error_status(
                    &received_str,
                    &completed_str,
                    source,
                    destination,
                    duration_ms,
                    status.code(),
                    status.message(),
                );
            }
        }
    }

    /// Converts a timestamp into a compact "HH:MM:SS" format.
    fn format_timestamp(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%H:%M:%S").to_string()
    }

    /// Logs an error response.
    fn log_error_status(
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
        code: Code,
        message: &str,
    ) {
        println!(
        "[{} - {}] Request from {}{}{} to {}{}{} ({} ms elapsed). Status: {}ERROR{} Code: {}, Message: {}",
        received_str,
        completed_str,
        YELLOW,
        source_str,
        RESET,
        CYAN,
        destination_str,
        RESET,
        duration_ms,
        RED,
        RESET,
        code,
        message
    );
    }
}

pub trait LoggableResponse {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    );
}

impl LoggableResponse for CommonResponse {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    ) {
        let status_color = if self.success { GREEN } else { RED };
        let status_text = if self.success { "SUCCESS" } else { "ERROR" };

        println!(
            "[{} - {}] Request from {}{}{} to {}{}{} ({} ms elapsed). Status: {}{}{} Message: {}",
            received_str,
            completed_str,
            YELLOW,
            source_str,
            RESET,
            CYAN,
            destination_str,
            RESET,
            duration_ms,
            status_color,
            status_text,
            RESET,
            self.message
        );
    }
}

impl LoggableResponse for Authentication {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    ) {
        println!(
            "[{} - {}] Request from {}{}{} to {}{}{} ({} ms elapsed). Status: {}SUCCESS{}",
            received_str,
            completed_str,
            YELLOW,
            source_str,
            RESET,
            CYAN,
            destination_str,
            RESET,
            duration_ms,
            GREEN,
            RESET
        );
    }
}

impl LoggableResponse for StatusResponse {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    ) {
        println!(
            "[{} - {}] Request from {}{}{} to {}{}{} ({} ms elapsed). Status: {}SUCCESS{}",
            received_str,
            completed_str,
            YELLOW,
            source_str,
            RESET,
            CYAN,
            destination_str,
            RESET,
            duration_ms,
            GREEN,
            RESET
        );
    }
}
