use crate::exec::exec_output;
use log::info;
use tss_esapi::{Context, TctiNameConf};

#[derive(Debug, Clone)]
pub struct RootLuks {
    pub luks_device: String,   // e.g. /dev/vda3 (device that holds the LUKS header)
    pub luks_uuid: String,     // luks UUID
    pub mapper_dev: String,   // e.g. vda3crypted
}

pub fn find_target_root_luks() -> Option<RootLuks> {
    // 1) get the device whose MOUNTPOINT is exactly /mnt
    //    NAME is absolute with -p; TYPE helps for logging.
    let out = exec_output(
        "lsblk",
        vec!["-nrp".into(), "-o".into(), "NAME,TYPE,MOUNTPOINT,PKNAME".into()],
    ).ok()?;
    let listing = String::from_utf8_lossy(&out.stdout);
    let mut mapper_dev = String::new();
    let mut parent_dev = String::new();

    for line in listing.lines() {
        // NAME TYPE MOUNTPOINT
        // e.g. "/dev/mapper/vdaXcrypted crypt /mnt"
        let mut it = line.split_whitespace();
        let name = it.next().unwrap_or("");
        let typ  = it.next().unwrap_or("");
        let mp    = it.next().unwrap_or("");
        let pk    = it.next().unwrap_or("");
        if (mp == "/mnt" || mp.starts_with("/mnt/")) && typ == "crypt" { // /mnt/home covers the BTRFS case where vdaXcrypted is shown to be mounted on /mnt/home on lsblk (because last mount on it)
            mapper_dev = name.to_string();
            parent_dev = pk.to_string();
            break;
        }
    }

    if mapper_dev.is_empty() {
        info!("find_target_root_luks: no device mounted at /mnt");
        return None;
    }
    if parent_dev.is_empty() || parent_dev == "-" {
        info!("Mapper {mapper_dev} has no PKNAME (maybe a loop device?)");
        return None;
    }
    info!("find_target_root_luks: start device {mapper_dev}");

    // get LUKS UUID from the parent_dev device
    let uuid_out = exec_output("cryptsetup", vec!["luksUUID".into(), parent_dev.clone()]).ok()?;
    let uuid = String::from_utf8_lossy(&uuid_out.stdout).trim().to_string();
    if !uuid_out.status.success() {
        info!("cryptsetup luksUUID failed for {parent_dev}");
        return None;
    } else if uuid.is_empty() {
        info!("cryptsetup luksUUID returned empty for {parent_dev}");
        return None;
    }

    Some(RootLuks { luks_device: parent_dev, luks_uuid: uuid, mapper_dev })
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