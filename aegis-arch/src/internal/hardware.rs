use shared::files;
use shared::info;
use shared::returncode_eval::files_eval;
use std::process::Command;
use std::thread::available_parallelism;

type Packages = Vec<&'static str>;
type Services = Vec<&'static str>;
type SetParams = Vec<(String, Vec<String>)>;

pub fn virt_check() -> (Packages, Services, SetParams) {
    let output = Command::new("systemd-detect-virt")
        .output()
        .expect("Failed to run systemd-detect-virt");

    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    result.pop(); //Removing the \n char from string

    let mut packages = Vec::new();
    let mut services = Vec::new();
    let mut set_params = Vec::new(); // To store the commands for file changes by sed

    if result == "oracle" {
        packages.push("virtualbox-guest-utils");
        services.push("vboxservice");
    } else if result == "vmware" {
        packages.extend(vec!["open-vm-tools", "xf86-video-vmware"]);
        services.extend(vec!["vmware-vmblock-fuse", "vmtoolsd"]);

        // Add the file change for vmware modules
        set_params.push((
            "Set vmware modules".to_string(),
            vec![
                "-i".to_string(),
                "-e".to_string(),
                "/^MODULES=()/ s/()/(vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx)/".to_string(),
                "-e".to_string(),
                "/^MODULES=([^)]*)/ {/vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx/! s/)/ vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx)/".to_string(),
                "/mnt/etc/mkinitcpio.conf".to_string(),
            ],
        ));
    } else if result == "qemu" || result == "kvm" {
        packages.extend(vec!["qemu-guest-agent", "spice-vdagent"]);
        services.push("qemu-guest-agent");
    } else if result == "microsoft" {
        packages.extend(vec!["hyperv", "xf86-video-fbdev"]);
        services.extend(vec![
            "hv_fcopy_daemon",
            "hv_kvp_daemon",
            "hv_vss_daemon",
        ]);

        // Add the file change for Hyper-V kernel parameter
        set_params.push((
            "Set hyperv kernel parameter".to_string(),
            vec![
                "-i".to_string(),
                "-e".to_string(),
                "/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/\"$/ video=hyperv_fb:3840x2160\"/g".to_string(),
                "/mnt/etc/default/grub".to_string(),
            ],
        ));
    }

    (packages, services, set_params) // Return packages, services, and file changes
}

pub fn set_cores() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    info!("The system has {} cores", default_parallelism_approx);
    if default_parallelism_approx > 1 {
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "#MAKEFLAGS=.*",
                &(format!("MAKEFLAGS=\"-j{}\"", default_parallelism_approx)),
            ),
            "Set available cores on MAKEFLAGS",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "#BUILDDIR=.*",
                "BUILDDIR=/tmp/makepkg",
            ),
            "Improving compilation times",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "COMPRESSXZ=\\(xz -c -z -\\)",
                "COMPRESSXZ=(xz -c -z - --threads=0)",
            ),
            "Changing the compression settings",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "COMPRESSZST=\\(zstd -c -z -q -\\)",
                "COMPRESSZST=(zstd -c -z -q - --threads=0)",
            ),
            "Changing the compression settings",
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "PKGEXT='.pkg.tar.xz'",
                "PKGEXT='.pkg.tar.zst'",
            ),
            "Changing the compression settings",
        );
    }
}

pub fn cpu_gpu_check(kernel: &str) -> Vec<&'static str> {
    let mut packages = Vec::new();

    // Detect CPU
    if cpu_detect().contains("Intel") {
        info!("Intel CPU detected.");
        packages.extend(vec!["intel-compute-runtime", "intel-ucode"]);
    } else if cpu_detect().contains("AMD") {
        info!("AMD CPU detected.");
        packages.push("amd-ucode");
    }

    // Detect GPU
    let gpudetect_output = Command::new("lspci")
        .arg("-k")
        .output()
        .expect("Failed to execute lspci -k command");

    let gpudetect = String::from_utf8_lossy(&gpudetect_output.stdout);
    let mut flag_gpu_found = false;

    if gpudetect.contains("AMD") {
        info!("AMD GPU detected.");
        packages.extend(vec!["xf86-video-amdgpu", "opencl-amd"]);
        flag_gpu_found = true;
    }

    if gpudetect.contains("ATI") && !gpudetect.contains("AMD") {
        info!("ATI GPU detected.");
        packages.push("opencl-mesa");
        flag_gpu_found = true;
    }

    if gpudetect.contains("NVIDIA") {
        info!("NVIDIA GPU detected.");

        if gpudetect.contains("GM107") || gpudetect.contains("GM108") || gpudetect.contains("GM200")
            || gpudetect.contains("GM204") || gpudetect.contains("GM206") || gpudetect.contains("GM20B") {
            info!("NV110 family (Maxwell)");
            flag_gpu_found = true;

            match kernel {
                "linux" => packages.push("nvidia"),
                "linux-lts" => packages.push("nvidia-lts"),
                _ => packages.push("nvidia-dkms"),
            }

            packages.push("nvidia-settings");
        }

        if gpudetect.contains("TU102") || gpudetect.contains("TU104") || gpudetect.contains("TU106")
            || gpudetect.contains("TU116") || gpudetect.contains("TU117") {
            info!("NV160 family (Turing)");
            flag_gpu_found = true;

            match kernel {
                "linux" => packages.push("nvidia-open"),
                _ => packages.push("nvidia-open-dkms"),
            }

            packages.push("nvidia-settings");
        }

        if gpudetect.contains("GK104") || gpudetect.contains("GK107") || gpudetect.contains("GK106")
            || gpudetect.contains("GK110") || gpudetect.contains("GK110B") || gpudetect.contains("GK208B")
            || gpudetect.contains("GK208") || gpudetect.contains("GK20A") || gpudetect.contains("GK210") {
            info!("NVE0 family (Kepler)");
            flag_gpu_found = true;
            packages.extend(vec!["nvidia-470xx-dkms", "nvidia-470xx-settings"]);
        }

        if gpudetect.contains("GF100") || gpudetect.contains("GF108") || gpudetect.contains("GF106")
            || gpudetect.contains("GF104") || gpudetect.contains("GF110") || gpudetect.contains("GF114")
            || gpudetect.contains("GF116") || gpudetect.contains("GF117") || gpudetect.contains("GF119") {
            info!("NVC0 family (Fermi)");
            flag_gpu_found = true;
            packages.extend(vec!["nvidia-390xx-dkms", "nvidia-390xx-settings"]);
        }

        if gpudetect.contains("G80") || gpudetect.contains("G84") || gpudetect.contains("G86")
            || gpudetect.contains("G92") || gpudetect.contains("G94") || gpudetect.contains("G96")
            || gpudetect.contains("G98") || gpudetect.contains("GT200") || gpudetect.contains("GT215")
            || gpudetect.contains("GT216") || gpudetect.contains("GT218") || gpudetect.contains("MCP77")
            || gpudetect.contains("MCP78") || gpudetect.contains("MCP79") || gpudetect.contains("MCP7A")
            || gpudetect.contains("MCP89") {
            info!("NV50 family (Tesla)");
            flag_gpu_found = true;
            packages.extend(vec!["nvidia-340xx-dkms", "nvidia-340xx-settings"]);
        }

        // For unrecognized NVIDIA families
        if !flag_gpu_found {
            packages.extend(vec!["nvidia-open-dkms", "nvidia-settings"]);
        }

        packages.extend(vec!["opencl-nvidia", "gwe", "nvtop"]);

        // Add envycontrol if hybrid GPU setup detected
        if gpudetect.contains("Intel") || gpudetect.contains("AMD") || gpudetect.contains("ATI") {
            packages.push("envycontrol");
        }
    }

    packages // Return the list of packages
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