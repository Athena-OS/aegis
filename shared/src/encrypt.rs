use crate::exec::exec_output;
use crate::returncode_eval::exec_eval_result;
use std::fs;

pub fn find_luks_partitions() -> Vec<(String, String)> {
    let mut luks_partitions = Vec::new();

    // Iterate over block devices in /dev
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            if let Some(device_name) = entry.file_name().to_str() {
                // Check if the device is a block device
                if device_name.starts_with("sd") || device_name.starts_with("nvme") || device_name.starts_with("mmcblk") {
                    let device_path = format!("/dev/{}", device_name);

                    // Check if the device is a LUKS partition
                    let output = exec_eval_result(
                        exec_output(
                            "cryptsetup",
                            vec![
                                String::from("luksDump"),
                                device_path.to_string(),
                            ],
                        ),
                        "Execute cryptsetup",
                    );
                        
                    // Check if the output contains LUKS header information
                    if output.status.success() {
                        if let Some(uuid) = parse_uuid_from_output(&output.stdout) {
                            luks_partitions.push((device_path, uuid));
                        }
                    }
                }
            }
        }
    }

    luks_partitions
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