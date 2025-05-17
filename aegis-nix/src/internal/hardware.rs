use shared::exec::exec_output;
use shared::files;
use shared::info;
use shared::returncode_eval::exec_eval_result;
use shared::returncode_eval::files_eval;
use std::process::{Command,Output};

pub fn virt_check() {
    let output_result = Command::new("systemd-detect-virt")
        .output(); // Directly call command
        // in baremetal, systemd-detect-virt returns exit status 1.
        // Here above I prevent it panics the application

    let output: Output = match output_result {
        Ok(out) => out,
        Err(e) => {
            panic!("Failed to execute systemd-detect-virt: {}", e);
        }
    };

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Allow "none" with exit code 1
    if output.status.code() != Some(0) && !(result == "none" && output.status.code() == Some(1)) {
        panic!(
            "Unexpected systemd-detect-virt failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if result == "oracle" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "virtualbox.guest.enable =.*",
                "virtualbox.guest.enable = lib.mkDefault true;",
            ),
            "enable virtualbox guest additions",
        );
    } else if result == "vmware" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "vmware.guest.enable =.*",
                "vmware.guest.enable = lib.mkDefault true;",
            ),
            "enable vmware guest additions",
        );
    } else if result == "qemu" || result == "kvm" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "spice-vdagentd.enable =.*",
                "spice-vdagentd.enable = lib.mkDefault true;",
            ),
            "enable spice vdagent",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "qemuGuest.enable =.*",
                "qemuGuest.enable = lib.mkDefault true;",
            ),
            "enable qemu guest additions",
        );
    } else if result == "microsoft" {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                "hypervGuest.enable =.*",
                "hypervGuest.enable = lib.mkDefault true;",
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
    let lscpu_output = exec_eval_result(
        exec_output(
            "lscpu",
            vec![]
        ),
        "Detect the CPU",
    );

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
