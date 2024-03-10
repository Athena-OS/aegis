use shared::files;
use shared::info;
use shared::returncode_eval::files_eval;
use std::process::Command;

pub fn virt_check() {
    let output = Command::new("systemd-detect-virt")
        .output()
        .expect("Failed to run systemd-detect-virt");

    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    result.pop(); //Removing the \n char from string

    if result == "oracle" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "virtualisation.virtualbox.guest.enable =.*",
                "virtualisation.virtualbox.guest.enable = true;",
            ),
            "enable virtualbox guest additions",
        );
    } else if result == "vmware" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "virtualisation.vmware.guest.enable =.*",
                "virtualisation.vmware.guest.enable = true;",
            ),
            "enable vmware guest additions",
        );
    } else if result == "qemu" || result == "kvm" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "services.spice-vdagentd.enable =.*",
                "services.spice-vdagentd.enable = true;",
            ),
            "enable spice vdagent",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "services.qemuGuest.enable =.*",
                "services.qemuGuest.enable = true;",
            ),
            "enable qemu guest additions",
        );
    } else if result == "microsoft" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "virtualisation.hypervGuest.enable =.*",
                "virtualisation.hypervGuest.enable = true;",
            ),
            "enable kvm guest additions",
        );
    }
}

pub fn cpu_check() {
    // Detect CPU
    if cpu_detect().contains("Intel") {
        info!("Intel CPU detected.");
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/default.nix",
                "cpu.intel.updateMicrocode =.*",
                "cpu.intel.updateMicrocode = true;",
            ),
            "enable intel ucode",
        );
    } else if cpu_detect().contains("AMD") {
        info!("AMD CPU detected.");
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/default.nix",
                "cpu.intel.updateMicrocode =.*",
                "cpu.amd.updateMicrocode = true;",
            ),
            "enable amd ucode",
        );
    }
}

fn cpu_detect() -> String {
    let lscpu_output = Command::new("lscpu")
        .output()
        .expect("Failed to run lscpu command");

    let lscpu_str = std::str::from_utf8(&lscpu_output.stdout)
        .expect("Failed to parse lscpu output as UTF-8");

    let vendor_id_line = lscpu_str
        .lines()
        .find(|line| line.starts_with("Vendor ID:"))
        .expect("Vendor ID not found in lscpu output");

    let vendor_id = vendor_id_line
        .split(':')
        .nth(1)
        .expect("Invalid format for Vendor ID in lscpu output")
        .trim();

    vendor_id.to_string()
}