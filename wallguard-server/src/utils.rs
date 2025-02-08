use crate::proto::wallguard::DeviceStatus;

/// Computes the MD5 hash of a given string.
///
/// # Arguments
/// * `input` - A string slice (`&str`) to be hashed.
///
/// # Returns
/// A `String` representing the MD5 hash in hexadecimal format.
pub fn digest(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}

/// Maps a given status string to its corresponding `DeviceStatus` enum.
///
/// This function converts the input string to lowercase and checks its prefix
/// to determine the appropriate `DeviceStatus` variant.
///
/// # Arguments
///
/// * `status` - A string slice that represents the status of a device.
///
/// # Returns
///
/// * A `DeviceStatus` variant corresponding to the given status.
pub fn map_status_value_to_enum(status: &str) -> DeviceStatus {
    let lowercase: String = status.to_lowercase();

    if lowercase.starts_with("draft") {
        DeviceStatus::DsDraft
    } else if lowercase.starts_with("active") {
        DeviceStatus::DsActive
    } else if lowercase.starts_with("archive") {
        DeviceStatus::DsArchived
    } else if lowercase.starts_with("delete") {
        DeviceStatus::DsDeleted
    } else {
        DeviceStatus::DsUnknown
    }
}
