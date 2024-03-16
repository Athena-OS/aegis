use crate::internal::hardware;
use shared::exec::exec;
use shared::files;
use shared::info;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;
use shared::strings::crash;
use std::path::PathBuf;

pub fn install_nix_config() {
    info!("Set nix channels.");
    exec_eval(
        exec(
            "nix-channel",
            vec![
                String::from("--add"),
                String::from("https://nixos.org/channels/nixos-23.11"),
                String::from("nixpkgs"),
            ],
        ),
        "Set nixpkgs nix channel",
    );
    exec_eval(
        exec(
            "nix-channel",
            vec![
                String::from("--update"),
            ],
        ),
        "Update nix channels",
    );
    std::fs::create_dir_all("/mnt/etc").unwrap();
    info!("Generate hardware configuration.");
    // nix-shell seems to work as non-sudo only by using --run; --command works only as sudo
    exec_eval(
        exec(
            "nix-shell",
            vec![
                String::from("-p"),
                String::from("nixos-install-tools"),
                String::from("--command"),
                String::from("nixos-generate-config --root /mnt"),
            ],
        ),
        "Run nixos-generate-config",
    );
    info!("Download latest Athena OS configuration.");
    exec_eval(
        exec(
            "curl",
            vec![
                String::from("-o"),
                String::from("/tmp/athena-nix.zip"),
                String::from("https://codeload.github.com/Athena-OS/athena-nix/zip/refs/heads/main"),
            ],
        ),
        "Getting latest Athena OS configuration.",
    );
    exec_eval(
        exec(
            "unzip",
            vec![
                String::from("/tmp/athena-nix.zip"),
                String::from("-d"),
                String::from("/tmp/"),
            ],
        ),
        "Extract Athena OS configuration archive.",
    );
    info!("Install Athena OS configuration.");
    exec_eval(
        exec(
            "cp",
            vec![
                String::from("-rf"),
                String::from("/tmp/athena-nix-main/nixos/home-manager"),
                String::from("/tmp/athena-nix-main/nixos/hosts"),
                String::from("/tmp/athena-nix-main/nixos/modules"),
                String::from("/tmp/athena-nix-main/nixos/pkgs"),
                String::from("/tmp/athena-nix-main/nixos/users"),
                String::from("/tmp/athena-nix-main/nixos/configuration.nix"),
                String::from("/tmp/athena-nix-main/nixos/default.nix"),
                String::from("/mnt/etc/nixos/"),
            ],
        ),
        "Move Athena OS configuration to /mnt/etc/nixos/.",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "/etc/nixos/hardware-configuration.nix",
            "./hardware-configuration.nix",
        ),
        "Set hardware-configuration path",
    );
    hardware::cpu_check();
    hardware::virt_check();
}

pub fn install_bootloader_efi(efidir: PathBuf) {
    info!("Set EFI Bootloader.");
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    info!("EFI bootloader installing at {}", efi_str);
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "systemd",
            "systemd",
        ),
        "Set EFI bootloader",
    );
}

pub fn install_bootloader_legacy(device: PathBuf) {
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    info!("Legacy bootloader installing at {}", device);
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/modules/boot/grub/default.nix",
            "/dev/sda",
            &device,
        ),
        "Set Legacy bootloader device",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "systemd",
            "grub",
        ),
        "Set Legacy bootloader",
    );
}

pub fn install_zram() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/modules/hardware/default.nix",
            "zramSwap.enable =.*",
            "zramSwap.enable = true;",
        ),
        "enable zram",
    );
}

pub fn install_flatpak() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "services.flatpak.enable =.*",
            "services.flatpak.enable = true;",
        ),
        "enable flatpak",
    );
}
