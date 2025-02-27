use crate::proto::wallguard::{ConfigStatus, DeviceStatus};

pub fn map_status_value_to_enum(status: String) -> DeviceStatus {
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

pub fn convert_status(status: ConfigStatus) -> String {
    match status {
        ConfigStatus::CsDraft => String::from("Draft"),
        ConfigStatus::CsApplied => String::from("Applied"),
        ConfigStatus::CsUndefined => String::from("Undefined"),
    }
}
