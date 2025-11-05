use crate::exec::exec_output;
use log::warn;
use std::path::Path;
use tss_esapi::{Context, TctiNameConf};

#[derive(Debug, Clone)]
pub struct RootLuks {
    pub luks_device: String,   // e.g. /dev/nvme0n1p2  (device that holds the LUKS header)
    pub luks_uuid: String,     // luks UUID
    pub mapper_name: String,   // e.g. cryptroot (from /dev/mapper/nvme0n1p2crypted)
}

/// Return the LUKS container that backs /mnt (target root), if any.
/// We do NOT scan the whole system; we just resolve the device mounted at /mnt and walk parents.
pub fn find_target_root_luks() -> Option<RootLuks> {
    // 1) Which block node backs /mnt?
    let src = exec_output("findmnt", vec!["-n".into(), "-o".into(), "SOURCE".into(), "/mnt".into()])
        .ok()?;
    let mut cur = String::from_utf8_lossy(&src.stdout).trim().to_string();
    if cur.is_empty() {
        warn!("find_target_root_luks: /mnt source is empty");
        return None;
    }

    // Normalize to absolute /dev path if needed
    if !cur.starts_with("/dev/") {
        // findmnt sometimes yields LABEL=… or UUID=…; resolve via lsblk
        if let Ok(out) = exec_output("lsblk", vec!["-no".into(), "PATH".into(), cur.clone()]) {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() { cur = s; }
        }
    }

    // 2) Walk parents with lsblk until we hit a LUKS container
    loop {
        // Is this node itself a LUKS container?
        if let Ok(ld) = exec_output("cryptsetup", vec!["isLuks".into(), cur.clone()])
            && ld.status.success() {
                // We found the LUKS header device (rare that FS sits directly here, but possible).
                let uuid = exec_output("cryptsetup", vec!["luksUUID".into(), cur.clone()])
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .filter(|s| !s.is_empty())?;
                // Try to get a live mapper name that corresponds to this LUKS
                let mapper = guess_mapper_name_for_backing(&cur).unwrap_or_else(|| {
                    // Fallback to conventional name (used by your installer)
                    format!("{}crypted", cur.trim_start_matches("/dev/"))
                });
                return Some(RootLuks { luks_device: cur, luks_uuid: uuid, mapper_name: mapper });
            }

        // Otherwise, move to its physical parent (PKNAME).
        let pk = exec_output("lsblk", vec!["-nr".into(), "-o".into(), "PKNAME".into(), cur.clone()])
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if pk.is_empty() || pk == cur || pk == "-" {
            // No more parents → not encrypted (at least not via LUKS)
            return None;
        }

        cur = if Path::new(&pk).is_absolute() { pk } else { format!("/dev/{pk}") };
    }
}

// Try to find an active /dev/mapper/* name that sits on top of `backing` (the LUKS header dev)
fn guess_mapper_name_for_backing(backing: &str) -> Option<String> {
    // Look for crypt children whose PKNAME equals `backing`
    if let Ok(out) = exec_output("lsblk", vec!["-rno".into(), "NAME,TYPE,PKNAME".into(), "-p".into()]) {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            // Example: "/dev/mapper/nvme0n1p2crypted crypt /dev/nvme0n1p2"
            let mut it = line.split_whitespace();
            let name = it.next().unwrap_or("");
            let typ  = it.next().unwrap_or("");
            let pk   = it.next().unwrap_or("");
            if typ == "crypt" && pk == backing && name.starts_with("/dev/mapper/") {
                return Some(name.trim_start_matches("/dev/mapper/").to_string());
            }
        }
    }
    None
}

pub fn tpm2_available_esapi() -> bool {
    // Prefer the kernel RM (/dev/tpmrm0); fall back to /dev/tpm0 if needed.
    // If omit the device path, the loader will try sensible defaults.
    let tcti = TctiNameConf::Device(Default::default());
    match Context::new(tcti) {
        Ok(mut _ctx) => true,   // we could also query properties here
        Err(_e) => false,
    }
}