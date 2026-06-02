use smbioslib::{SMBiosSystemInformation, table_load_from_device};

/// Retrieves the UUID of the device by reading the SMBIOS (System Management BIOS) table.
/// Falls back to platform-specific tools if the SMBIOS table cannot be read directly.
///
/// # Returns
/// - `Some<String>` containing the UUID if it can be successfully retrieved.
/// - `None` if all methods fail or if the UUID field is not present.
pub fn retrieve_device_uuid() -> Option<String> {
    // Primary: read directly from SMBIOS (works on Linux/FreeBSD/Windows).
    let uuid = table_load_from_device().ok().and_then(|table| {
        table
            .find_map(|value: SMBiosSystemInformation| value.uuid())
            .map(|uuid| uuid.to_string())
    });

    if uuid.is_some() {
        return uuid;
    }

    // macOS: SMBIOS device files are unavailable; use IORegistry instead.
    #[cfg(target_os = "macos")]
    return retrieve_device_uuid_via_ioreg();

    // Linux / FreeBSD fallback: invoke dmidecode.
    #[cfg(not(target_os = "macos"))]
    retrieve_device_uuid_via_dmidecode()
}

/// Reads the hardware UUID from the macOS IORegistry via `ioreg`.
///
/// `ioreg -rd1 -c IOPlatformExpertDevice` prints a line like:
///   "IOPlatformUUID" = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
#[cfg(target_os = "macos")]
fn retrieve_device_uuid_via_ioreg() -> Option<String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;

    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            // Line format:  "IOPlatformUUID" = "XXXXXXXX-…"
            // Split on '"' and the UUID is the 4th token (index 3).
            let parts: Vec<&str> = line.splitn(5, '"').collect();
            if let Some(uuid) = parts.get(3) {
                let uuid = uuid.trim().to_string();
                if !uuid.is_empty() {
                    return Some(uuid);
                }
            }
        }
    }

    None
}

/// Attempts to retrieve the device UUID by spawning `dmidecode`.
#[cfg(not(target_os = "macos"))]
fn retrieve_device_uuid_via_dmidecode() -> Option<String> {
    let output = std::process::Command::new("dmidecode")
        .args(["-s", "system-uuid"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let uuid = String::from_utf8(output.stdout).ok()?.trim().to_string();

    if uuid.is_empty() { None } else { Some(uuid) }
}
