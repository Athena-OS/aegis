use crate::exec::exec_output;
use log::info;
use std::fs;
use tss_esapi::{Context, TctiNameConf};

pub fn tpm2_available_esapi() -> bool {
    // Prefer the kernel RM (/dev/tpmrm0); fall back to /dev/tpm0 if needed.
    // If omit the device path, the loader will try sensible defaults.
    let tcti = TctiNameConf::Device(Default::default());
    match Context::new(tcti) {
        Ok(mut _ctx) => true,   // we could also query properties here
        Err(_e) => false,
    }
}

pub fn find_luks_partitions() -> (Vec<(String, String)>, bool) {
    let mut luks_partitions = Vec::new();

    // Iterate over block devices in /dev
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            if let Some(device_name) = entry.file_name().to_str() {
                // Check if the device is a likely block device name
                if device_name.starts_with("sd")
                    || device_name.starts_with("vd")
                    || device_name.starts_with("nvme")
                    || device_name.starts_with("mmcblk")
                {
                    let device_path = format!("/dev/{device_name}");

                    // Check if the device is a LUKS partition
                    match exec_output(
                        "cryptsetup",
                        vec![String::from("luksDump"), device_path.to_string()],
                    ) {
                        Ok(output) => {
                            if output.status.success()
                                && let Some(uuid) = parse_uuid_from_output(&output.stdout) {
                                    info!("LUKS partition found: {device_path} - {uuid}");
                                    luks_partitions.push((device_path, uuid));
                                }
                        }
                        Err(err) => {
                            info!("LUKS dump failed: {err}");
                            // keep going
                        }
                    }
                }
            }
        }
    }

    let encrypt_check = !luks_partitions.is_empty();
    (luks_partitions, encrypt_check)
}

fn parse_uuid_from_output(output: &[u8]) -> Option<String> {
    let output_str = String::from_utf8_lossy(output);

    for line in output_str.lines() {
        if line.starts_with("UUID:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(uuid) = parts.get(1) {
                return Some(uuid.to_string());
            }
        }
    }

    None
}