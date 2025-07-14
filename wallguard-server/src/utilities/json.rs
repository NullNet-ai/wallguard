use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::Value as JsonValue;

/// Attempts to parse a JSON string into a `serde_json::Value`.
///
/// Returns a `Result` containing the parsed JSON or an error wrapped with location context.
pub fn parse_string(data: &str) -> Result<JsonValue, Error> {
    serde_json::from_str::<JsonValue>(data).handle_err(location!())
}

/// Extracts the first element from a JSON array.
///
/// Returns a reference to the first `JsonValue` if available, or an error if the input is not an array or is empty.
pub fn first_element_from_array(value: &JsonValue) -> Result<JsonValue, Error> {
    value
        .as_array()
        .and_then(|arr| arr.first())
        .cloned()
        .ok_or("Operation failed")
        .handle_err(location!())
}
