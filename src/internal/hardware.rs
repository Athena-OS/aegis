use std::process::Command;
use std::thread::available_parallelism;
use crate::args::PackageManager;
use crate::internal::exec::*;
use crate::internal::*;
use crate::internal::services::enable_service;

pub fn virt_check() {
    let output = Command::new("systemd-detect-virt")
        .output()
        .expect("Failed to run systemd-detect-virt");

    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    result.pop(); //Removing the \n char from string

    if result == "oracle" {
        install(PackageManager::Pacman, vec!["virtualbox-guest-utils"]);
        enable_service("vboxservice");
    } else if result == "vmware" {
        install(PackageManager::Pacman, vec!["open-vm-tools", "xf86-video-vmware"]);
        enable_service("vmware-vmblock-fuse");
        enable_service("vmtoolsd");
        enable_service("mnt-hgfs.mount");

        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^MODULES*/ s/\"$/ vsock vmw_vsock_vmci_transport vmw_balloon vmw_vmci vmwgfx\"/g'"),
                    String::from("/etc/mkinitcpio.conf"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Set vmware kernel parameter",
        );
    } else if result == "qemu" || result == "kvm" {
        install(PackageManager::Pacman, vec!["qemu-guest-agent"]);
        enable_service("qemu-guest-agent");
    } else if result == "microsoft" {
        install(PackageManager::Pacman, vec!["hyperv", "xf86-video-fbdev"]);
        enable_service("hv_fcopy_daemon");
        enable_service("hv_kvp_daemon");
        enable_service("hv_vss_daemon");

        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/\"$/ video=hyperv_fb:3840x2160\"/g'"),
                    String::from("/etc/default/grub"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Set hyperv kernel parameter",
        );
    }
}

pub fn set_cores() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    log::info!("The system has {} cores", default_parallelism_approx);
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

pub fn cpu_gpu_check(kernel: &str) {
    // Detect CPU
    
    if cpu_detect().contains("Intel") {
        log::info!("Intel CPU detected.");
        install(PackageManager::Pacman, vec!["intel-compute-runtime", "intel-ucode"]);
    } else if cpu_detect().contains("AMD") {
        log::info!("AMD CPU detected.");
        install(PackageManager::Pacman, vec!["amd-ucode"]);
    }
    
    // Detect GPU
    let gpudetect_output = Command::new("lspci")
        .arg("-k")
        .output()
        .expect("Failed to execute lspci -k command");
    
    let gpudetect = String::from_utf8_lossy(&gpudetect_output.stdout);
    let mut flag_gpu_found = false;
    
    if gpudetect.contains("AMD") {
        log::info!("AMD GPU detected.");
        install(PackageManager::Pacman, vec!["xf86-video-amdgpu", "opencl-amd"]);
        flag_gpu_found = true;
    }
    
    if gpudetect.contains("ATI") && !gpudetect.contains("AMD") {
        log::info!("ATI GPU detected.");
        install(PackageManager::Pacman, vec!["opencl-mesa"]);
        flag_gpu_found = true;
    }
    
    if gpudetect.contains("NVIDIA") {
        log::info!("NVIDIA GPU detected.");

        // https://wiki.archlinux.org/title/NVIDIA#Installation
        // https://nouveau.freedesktop.org/CodeNames.html
        
        if gpudetect.contains("GM107") || gpudetect.contains("GM108")
            || gpudetect.contains("GM200") || gpudetect.contains("GM204")
            || gpudetect.contains("GM206") || gpudetect.contains("GM20B")
        {
            log::info!("NV110 family (Maxwell)");
            flag_gpu_found = true;
            
            if kernel == "linux" {
                install(PackageManager::Pacman, vec!["nvidia"]);
            } else if kernel == "linux-lts" {
                install(PackageManager::Pacman, vec!["nvidia-lts"]);
            } else {
                install(PackageManager::Pacman, vec!["nvidia-dkms"]);
            }
            
            install(PackageManager::Pacman, vec!["nvidia-settings"]);
        }

        if gpudetect.contains("TU102") || gpudetect.contains("TU104")
            || gpudetect.contains("TU106") || gpudetect.contains("TU116")
            || gpudetect.contains("TU117")
        {
            log::info!("NV160 family (Turing)");
            flag_gpu_found = true;
            if kernel == "linux" {
                install(PackageManager::Pacman, vec!["nvidia-open"]);
            } else {
                install(PackageManager::Pacman, vec!["nvidia-open-dkms"]);
            }
            
            install(PackageManager::Pacman, vec!["nvidia-settings"]);
        }

        if gpudetect.contains("GK104") || gpudetect.contains("GK107")
            || gpudetect.contains("GK106") || gpudetect.contains("GK110")
            || gpudetect.contains("GK110B") || gpudetect.contains("GK208B")
            || gpudetect.contains("GK208") || gpudetect.contains("GK20A")
            || gpudetect.contains("GK210")
        {
            log::info!("NVE0 family (Kepler)");
            flag_gpu_found = true;
            install(PackageManager::Pacman, vec!["nvidia-470xx-dkms", "nvidia-470xx-settings"]);
        }

        if gpudetect.contains("GF100") || gpudetect.contains("GF108")
            || gpudetect.contains("GF106") || gpudetect.contains("GF104")
            || gpudetect.contains("GF110") || gpudetect.contains("GF114")
            || gpudetect.contains("GF116") || gpudetect.contains("GF117")
            || gpudetect.contains("GF119")
        {
            log::info!("NVC0 family (Fermi)");
            flag_gpu_found = true;
            install(PackageManager::Pacman, vec!["nvidia-390xx-dkms", "nvidia-390xx-settings"]);
        }

        if gpudetect.contains("G80") || gpudetect.contains("G84")
            || gpudetect.contains("G86") || gpudetect.contains("G92")
            || gpudetect.contains("G94") || gpudetect.contains("G96")
            || gpudetect.contains("G98") || gpudetect.contains("GT200")
            || gpudetect.contains("GT215") || gpudetect.contains("GT216")
            || gpudetect.contains("GT218") || gpudetect.contains("MCP77")
            || gpudetect.contains("MCP78") || gpudetect.contains("MCP79")
            || gpudetect.contains("MCP7A") || gpudetect.contains("MCP89")
        {
            log::info!("NV50 family (Tesla)");
            flag_gpu_found = true;
            install(PackageManager::Pacman, vec!["nvidia-340xx-dkms", "nvidia-340xx-settings"]);
        }
        
        // For not recognized families
        
        if !flag_gpu_found {
            install(PackageManager::Pacman, vec!["nvidia-open-dkms", "nvidia-settings"]);
        }
        
        install(PackageManager::Pacman, vec!["opencl-nvidia", "gwe", "nvtop"]);
        
        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^MODULES*/ s/\"$/ nvidia nvidia_modeset nvidia_uvm nvidia_drm\"/g'"),
                    String::from("/etc/mkinitcpio.conf"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Enable NVIDIA GPU modules",
        );
        exec_eval(
            exec_chroot(
                "sed",
                vec![
                    String::from("-in"),
                    String::from("'/^GRUB_CMDLINE_LINUX_DEFAULT*/ s/\"$/ nvidia-drm.modeset=1\"/g'"),
                    String::from("/etc/default/grub"), //In chroot we don't need to specify /mnt
                ],
            ),
            "Enable NVIDIA GPU kernel paramater",
        );
        
        if gpudetect.contains("Intel") || gpudetect.contains("AMD") || gpudetect.contains("ATI") {
            install(PackageManager::Pacman, vec!["envycontrol", "nvidia-exec"]);
        }
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