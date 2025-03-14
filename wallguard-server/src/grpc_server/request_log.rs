use chrono::{DateTime, Utc};
use nullnet_liberror::Error;
use tonic::{Request, Response};

use crate::proto::wallguard::{
    Authentication, CommonResponse, ControlChannelResponse, HeartbeatResponse, StatusResponse,
};

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
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string())
    }

    /// Unified log response function for different response types.
    pub(crate) fn log_response<T>(
        response: &Result<Response<T>, Error>,
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
            Err(error) => {
                Self::log_error_status(
                    &received_str,
                    &completed_str,
                    source,
                    destination,
                    duration_ms,
                    error,
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
        error: &Error,
    ) {
        log::error!(
            "[{received_str} - {completed_str}] Request from {YELLOW}{source_str}{RESET} to {CYAN}{destination_str}{RESET} ({duration_ms} ms elapsed). Status: {RED}ERROR{RESET}: {error:?}",
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
        log::info!(
            "[{} - {}] Request from {}{}{} to {}{}{} ({} ms elapsed). {}SUCCESS{} Message: {}",
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
        log_success_common(
            received_str,
            completed_str,
            source_str,
            destination_str,
            duration_ms,
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
        log_success_common(
            received_str,
            completed_str,
            source_str,
            destination_str,
            duration_ms,
        );
    }
}

impl LoggableResponse for HeartbeatResponse {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    ) {
        log_success_common(
            received_str,
            completed_str,
            source_str,
            destination_str,
            duration_ms,
        );
    }
}

impl LoggableResponse for ControlChannelResponse {
    fn log_success(
        &self,
        received_str: &str,
        completed_str: &str,
        source_str: &str,
        destination_str: &str,
        duration_ms: i64,
    ) {
        log_success_common(
            received_str,
            completed_str,
            source_str,
            destination_str,
            duration_ms,
        );
    }
}

fn log_success_common(
    received_str: &str,
    completed_str: &str,
    source_str: &str,
    destination_str: &str,
    duration_ms: i64,
) {
    log::info!(
        "[{received_str} - {completed_str}] Request from {YELLOW}{source_str}{RESET} to {CYAN}{destination_str}{RESET} ({duration_ms} ms elapsed). Status: {GREEN}SUCCESS{RESET}",
    );
}
