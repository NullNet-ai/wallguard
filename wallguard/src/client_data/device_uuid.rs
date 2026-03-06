use smbioslib::{SMBiosSystemInformation, table_load_from_device};

/// Retrieves the UUID of the device by reading the SMBIOS (System Management BIOS) table.
/// Falls back to calling `dmidecode` if the SMBIOS table cannot be read.
///
/// # Returns
/// - `Some<String>` containing the UUID if it can be successfully retrieved.
/// - `None` if both methods fail or if the UUID field is not present.
pub fn retrieve_device_uuid() -> Option<String> {
    // Primary: read directly from SMBIOS
    let uuid = table_load_from_device()
        .ok()
        .and_then(|table| {
            table.find_map(|value: SMBiosSystemInformation| value.uuid())
                .map(|uuid| uuid.to_string())
        });

    if uuid.is_some() {
        return uuid;
    }

    // Fallback: invoke dmidecode
    retrieve_device_uuid_via_dmidecode()
}

/// Attempts to retrieve the device UUID by spawning `dmidecode`.
fn retrieve_device_uuid_via_dmidecode() -> Option<String> {
    let output = std::process::Command::new("dmidecode")
        .args(["-s", "system-uuid"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let uuid = String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .to_string();

    if uuid.is_empty() {
        None
    } else {
        Some(uuid)
    }
}