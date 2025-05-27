use crate::utilities;
use libudev::{Context, Enumerator};

fn list_disks_with_serials() -> Vec<(String, String)> {
    let context = match Context::new() {
        Ok(ctx) => ctx,
        Err(_) => return Vec::new(),
    };

    let mut enumerator = match Enumerator::new(&context) {
        Ok(en) => en,
        Err(_) => return Vec::new(),
    };

    if enumerator.match_subsystem("block").is_err() {
        return Vec::new();
    }

    if enumerator.match_property("DEVTYPE", "disk").is_err() {
        return Vec::new();
    }

    let devices = match enumerator.scan_devices() {
        Ok(devices) => devices,
        Err(_) => return Vec::new(),
    };

    let mut disks = Vec::new();

    for device in devices {
        if let Some(devnode) = device.devnode() {
            if let Some(serial) = device.property_value("ID_SERIAL") {
                disks.push((
                    devnode.to_string_lossy().into_owned(),
                    serial.to_string_lossy().into(),
                ));
            }
        }
    }

    disks
}

pub fn disks_fingerprint() -> Option<String> {
    let disks = list_disks_with_serials();
    
    if disks.is_empty() {
        return None;
    }

    let raw = disks
        .iter()
        .map(|(devnode, serial)| format!("{}|{}", devnode, serial))
        .collect::<Vec<_>>()
        .join("\n");

    Some(utilities::hash::sha256_digest_hex(&raw))
}
