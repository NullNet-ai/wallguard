use crate::utilities;
use smbioslib::{
    table_load_from_device, SMBiosBaseboardInformation, SMBiosProcessorInformation,
    SMBiosSystemInformation,
};

macro_rules! push_smbiosstring {
    ($vec:expr, $expr:expr) => {
        if $expr.is_ok() {
            let value = format!("{}", $expr.as_ref().unwrap());
            if !value.is_empty() {
                $vec.push(value);
            }
        }
    };
}

fn collect_smbios_identifiers() -> Option<String> {
    let data = table_load_from_device().ok()?;

    let mut parts = Vec::new();

    if let Some(sysinfo) = data.find_map(|val: SMBiosSystemInformation| Some(val)) {
        if let Some(uuid) = sysinfo.uuid() {
            parts.push(format!("{}\n", uuid));
        }

        push_smbiosstring!(parts, sysinfo.manufacturer());
        push_smbiosstring!(parts, sysinfo.product_name());
        push_smbiosstring!(parts, sysinfo.version());
        push_smbiosstring!(parts, sysinfo.serial_number());
        push_smbiosstring!(parts, sysinfo.family());
        push_smbiosstring!(parts, sysinfo.sku_number());
    }

    if let Some(boardinfo) = data.find_map(|val: SMBiosBaseboardInformation| Some(val)) {
        push_smbiosstring!(parts, boardinfo.manufacturer());
        push_smbiosstring!(parts, boardinfo.serial_number());
        push_smbiosstring!(parts, boardinfo.version());
        push_smbiosstring!(parts, boardinfo.product());
    }

    if let Some(procinfo) = data.find_map(|val: SMBiosProcessorInformation| Some(val)) {
        push_smbiosstring!(parts, procinfo.asset_tag());
        push_smbiosstring!(parts, procinfo.part_number());
        push_smbiosstring!(parts, procinfo.processor_manufacturer());
        push_smbiosstring!(parts, procinfo.processor_version());
        push_smbiosstring!(parts, procinfo.serial_number());
        push_smbiosstring!(parts, procinfo.socket_designation());
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

pub fn system_fingerprint() -> Option<String> {
    let raw_id = collect_smbios_identifiers()?;
    Some(utilities::hash::sha256_digest_hex(&raw_id))
}
