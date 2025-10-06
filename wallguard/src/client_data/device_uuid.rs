use smbioslib::{SMBiosSystemInformation, table_load_from_device};

/// Retrieves the UUID of the device by reading the SMBIOS (System Management BIOS) table.
///
/// # Returns
/// - `Some<String>` containing the UUID if it can be successfully retrieved.
/// - `None` if the SMBIOS table cannot be read or if the UUID field is not present.
pub fn retrieve_device_uuid() -> Option<String> {
    table_load_from_device()
        .ok()?
        .find_map(|value: SMBiosSystemInformation| value.uuid())
        .map(|uuid| uuid.to_string())
}
