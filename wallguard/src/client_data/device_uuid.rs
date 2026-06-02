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

    // Linux: try sysfs and machine-id before dmidecode (work without root).
    #[cfg(target_os = "linux")]
    {
        if let Some(uuid) = retrieve_device_uuid_via_sysfs() {
            return Some(uuid);
        }
        if let Some(uuid) = retrieve_device_uuid_via_dmidecode() {
            return Some(uuid);
        }
        return retrieve_device_uuid_via_machine_id();
    }

    // FreeBSD fallback: invoke dmidecode.
    #[cfg(target_os = "freebsd")]
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

/// Reads the hardware UUID from `/sys/class/dmi/id/product_uuid` (world-readable on Linux ≥ 4.14).
#[cfg(target_os = "linux")]
fn retrieve_device_uuid_via_sysfs() -> Option<String> {
    let uuid = std::fs::read_to_string("/sys/class/dmi/id/product_uuid")
        .ok()?
        .trim()
        .to_string();
    if uuid.is_empty() { None } else { Some(uuid) }
}

/// Reads `/etc/machine-id` as a last-resort UUID (always readable, stable per machine).
#[cfg(target_os = "linux")]
fn retrieve_device_uuid_via_machine_id() -> Option<String> {
    let id = std::fs::read_to_string("/etc/machine-id")
        .ok()?
        .trim()
        .to_string();
    if id.is_empty() { None } else { Some(id) }
}

/// Attempts to retrieve the device UUID by spawning `dmidecode`.
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
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
