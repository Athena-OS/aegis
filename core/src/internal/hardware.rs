use log::info;
use shared::args::{is_arch, is_fedora, is_nix};
use shared::files;
use shared::exec::exec_output;
use shared::returncode_eval::{exec_eval_result, files_eval};
use std::process::{Command, Output};
use std::thread::available_parallelism;

type Packages = Vec<&'static str>;
type Services = Vec<&'static str>;
type SetParams = Vec<(String, Vec<String>)>;

pub fn virt_check() -> (Packages, Services, SetParams) {
    let output_result = Command::new("systemd-detect-virt")
        .output(); // Directly call command
        // in baremetal, when no virtualization is detected, systemd-detect-virt returns exit status 1.
        // So we use directly Command::new to prevent it panics the application

    let output: Output = match output_result {
        Ok(out) => out,
        Err(e) => {
            panic!("Failed to execute systemd-detect-virt: {e}");
        }
    };

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    info!("Virtualization detected: {result}");

    let mut packages = Vec::new();
    let mut services = Vec::new();
    let mut set_params = Vec::new(); // To store the commands for file changes by sed

    match result.as_str() {
        "oracle" => {
            if is_arch() {
                packages.push("virtualbox-guest-utils");
                services.push("vboxservice");
            } else if is_fedora() {
                packages.push("virtualbox-guest-additions");
                services.push("vboxservice");
            } else if is_nix() {
                files_eval(
                    files::sed_file(
                        "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                        "virtualbox.guest.enable =.*",
                        "virtualbox.guest.enable = lib.mkDefault true;",
                    ),
                    "enable virtualbox guest additions",
                );
            }
        }

        "vmware" => {
            if is_arch() {
                packages.extend(["open-vm-tools"]);
                services.extend(["vmware-vmblock-fuse", "vmtoolsd"]);

                // Add the mkinitcpio MODULES edits for VMware on Arch
                set_params.push((
                    "Set vmware modules".to_string(),
                    vec![
                        "-i".into(),
                        "-e".into(),
                        "/^MODULES=()/ s/()/(vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx)/".into(),
                        "-e".into(),
                        "/^MODULES=([^)]*)/ {/vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx/! s/)/ vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx)/".into(),
                        "/mnt/etc/mkinitcpio.conf".into(),
                    ],
                ));
            } else if is_fedora() {
                packages.extend(["open-vm-tools", "xorg-x11-drv-vmware"]);
                services.push("vmtoolsd");
            } else if is_nix() {
                files_eval(
                    files::sed_file(
                        "/mnt/etc/nixos/modules/hardware/virtualization/guest.nix",
                        "vmware.guest.enable =.*",
                        "vmware.guest.enable = lib.mkDefault true;",
                    ),
                    "enable vmware guest additions",
                );                
            }
        }

        "qemu" | "kvm" => {
            if !is_nix() {
                packages.extend(["qemu-guest-agent", "spice-vdagent"]);
                services.push("qemu-guest-agent");
            } else {
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
            }
        }

        "microsoft" => {
            if !is_nix() {
                if is_arch() {
                    packages.extend(["hyperv", "xf86-video-fbdev"]);
                    services.extend(["hv_fcopy_daemon", "hv_kvp_daemon", "hv_vss_daemon"]);
                } else if is_fedora() {
                    packages.push("hyperv-tools");
                }
                set_params.push((
                    "Set hyperv kernel parameter".to_string(),
                    vec![
                        "-i".into(),
                        "-e".into(),
                        "/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/\"$/ video=hyperv_fb:3840x2160\"/g".into(),
                        "/mnt/etc/default/grub".into(),
                    ],
                ));
            }
            else {
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

        "none" => info!("Running on bare metal."),
        _ => info!("Unknown virtualization type: {result}"),
    }

    (packages, services, set_params) // Return packages, services, and file changes
}

pub fn set_cores() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    info!("The system has {default_parallelism_approx} cores");
    if default_parallelism_approx > 1 {
        files_eval(
            files::sed_file(
                "/mnt/etc/makepkg.conf",
                "#MAKEFLAGS=.*",
                &(format!("MAKEFLAGS=\"-j{default_parallelism_approx}\"")),
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

pub fn cpu_check() -> Vec<&'static str> {
    let mut packages: Vec<&'static str> = Vec::new();
    // -------- CPU --------
    let cpu = cpu_detect();
    if cpu.contains("Intel") {
        info!("Intel CPU detected.");
        if is_arch() {
            packages.push("intel-ucode");
            packages.push("intel-compute-runtime");
        } else if is_fedora() {
            packages.push("microcode_ctl");
            packages.push("intel-compute-runtime");
        } else if is_nix() {
            files_eval(
                files::sed_file(
                    "/mnt/etc/nixos/modules/hardware/default.nix",
                    "cpu.intel.updateMicrocode =.*",
                    "cpu.intel.updateMicrocode = true;",
                ),
                "enable intel ucode",
            );            
        }
    } else if cpu.contains("AMD") {
        info!("AMD CPU detected.");
        if is_arch() {
            packages.push("amd-ucode");
        } else if is_fedora() {
            packages.push("amd-ucode-firmware");
        } else if is_nix() {
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
    packages
}

pub fn gpu_check(kernel: &str) -> Vec<&'static str> {
    let mut packages: Vec<&'static str> = Vec::new();

    // -------- GPU --------
    let gpudetect_output = exec_eval_result(
        exec_output("lspci", vec![String::from("-k")]),
        "Detect the GPU",
    );
    let gpudetect = String::from_utf8_lossy(&gpudetect_output.stdout);
    
    // AMD
    if gpudetect.contains("AMD") {
        info!("AMD GPU detected.");
        if is_arch() {
            packages.extend(["xf86-video-amdgpu", "opencl-amd"]);
        } else if is_fedora() {
            packages.extend(["xorg-x11-drv-amdgpu", "amd-gpu-firmware"]);
        }
    }
    
    // ATI (legacy, not reporting AMD)
    if gpudetect.contains("ATI") && !gpudetect.contains("AMD") {
        info!("ATI GPU detected.");
        if is_arch() {
            packages.push("opencl-mesa");
        } else if is_fedora() {
            packages.push("mesa-libOpenCL");
        }
    }
    
    // NVIDIA
    if gpudetect.contains("NVIDIA") {
        info!("NVIDIA GPU detected.");
    
        if is_arch() {
            // Family-specific handling (your Arch logic)
            let mut matched_family = false;
        
            if gpudetect.contains("GM107") || gpudetect.contains("GM108") || gpudetect.contains("GM200")
                || gpudetect.contains("GM204") || gpudetect.contains("GM206") || gpudetect.contains("GM20B")
            {
                info!("NV110 family (Maxwell)");
                matched_family = true;
                match kernel {
                    "linux" => packages.push("nvidia"),
                    "linux-lts" => packages.push("nvidia-lts"),
                    _ => packages.push("nvidia-dkms"),
                }
                packages.push("nvidia-settings");
            }
        
            if gpudetect.contains("TU102") || gpudetect.contains("TU104") || gpudetect.contains("TU106")
                || gpudetect.contains("TU116") || gpudetect.contains("TU117")
            {
                info!("NV160 family (Turing)");
                matched_family = true;
                match kernel {
                    "linux" => packages.push("nvidia-open"),
                    _ => packages.push("nvidia-open-dkms"),
                }
                packages.push("nvidia-settings");
            }
        
            if gpudetect.contains("GK104") || gpudetect.contains("GK107") || gpudetect.contains("GK106")
                || gpudetect.contains("GK110") || gpudetect.contains("GK110B") || gpudetect.contains("GK208B")
                || gpudetect.contains("GK208") || gpudetect.contains("GK20A") || gpudetect.contains("GK210")
            {
                info!("NVE0 family (Kepler)");
                matched_family = true;
                packages.extend(["nvidia-470xx-dkms", "nvidia-470xx-settings"]);
            }
        
            if gpudetect.contains("GF100") || gpudetect.contains("GF108") || gpudetect.contains("GF106")
                || gpudetect.contains("GF104") || gpudetect.contains("GF110") || gpudetect.contains("GF114")
                || gpudetect.contains("GF116") || gpudetect.contains("GF117") || gpudetect.contains("GF119")
            {
                info!("NVC0 family (Fermi)");
                matched_family = true;
                packages.extend(["nvidia-390xx-dkms", "nvidia-390xx-settings"]);
            }
        
            if gpudetect.contains("G80") || gpudetect.contains("G84") || gpudetect.contains("G86")
                || gpudetect.contains("G92") || gpudetect.contains("G94") || gpudetect.contains("G96")
                || gpudetect.contains("G98") || gpudetect.contains("GT200") || gpudetect.contains("GT215")
                || gpudetect.contains("GT216") || gpudetect.contains("GT218") || gpudetect.contains("MCP77")
                || gpudetect.contains("MCP78") || gpudetect.contains("MCP79") || gpudetect.contains("MCP7A")
                || gpudetect.contains("MCP89")
            {
                info!("NV50 family (Tesla)");
                matched_family = true;
                packages.extend(["nvidia-340xx-dkms", "nvidia-340xx-settings"]);
            }
        
            if !matched_family {
                packages.extend(["nvidia-open-dkms", "nvidia-settings"]);
            }
        
            // Common extras on Arch
            packages.extend(["opencl-nvidia", "gwe", "nvtop"]);
        
            // Hybrid GPU setup? add envycontrol on Arch like your original
            if gpudetect.contains("Intel") || gpudetect.contains("AMD") || gpudetect.contains("ATI") {
                packages.push("envycontrol");
            }
        } else if is_fedora() {
            // Fedora path (your simpler logic)
            packages.push("nvidia-gpu-firmware");
            packages.extend(["gwe", "nvtop"]);
        }
    }
    packages
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
