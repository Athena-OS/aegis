use crate::internal::hardware;
use crate::internal::install::install;
use crate::internal::services::enable_service;
use log::{info, error};
use shared::args::{Base, distro_base, ExtendIntoString, InstallMode, PackageManager, is_arch};
use shared::exec::{exec, exec_archchroot, exec_output};
use shared::encrypt::{find_luks_partitions, tpm2_available_esapi};
use shared::files;
use shared::returncode_eval::{exec_eval, exec_eval_result, files_eval};
use std::{fs, path::PathBuf};

pub fn install_packages(mut packages: Vec<String>, kernel: &str) -> i32 {
    let (kernel_to_install, kernel_headers_to_install) = (kernel, format!("{kernel}-headers"));
    let arch_base_pkg: Vec<&str> = vec![
        // Kernel
        kernel_to_install,
        &kernel_headers_to_install,
        "linux-hardened",
        "linux-hardened-headers",
        // Base Arch
        "base",
        "glibc-locales", // Prebuilt locales to prevent locales warning message during the pacstrap install of base metapackage
        // Repositories
        "athena-mirrorlist",
        "chaotic-mirrorlist",
        "rate-mirrors",
        "archlinux-keyring",
        "athena-keyring",
        "chaotic-keyring",
    ];

    packages.extend_into(arch_base_pkg);

    if tpm2_available_esapi() {
        packages.extend_into(["tpm2-tools"]);
    }

    /***** CHECK IF BTRFS *****/
    let (fstype, _fs_uuid) = detect_root_fs_info();
    if fstype == "btrfs" {
        packages.extend_into(["btrfs-progs"]);
    }
    info!("Root partition is {fstype}");

    let (virt_packages, virt_services, virt_params) = hardware::virt_check();
    let cpu_packages = hardware::cpu_check();
    let gpu_packages = hardware::gpu_check(kernel_to_install);
    packages.extend_into(virt_packages);
    packages.extend_into(cpu_packages);
    packages.extend_into(gpu_packages);

    std::fs::create_dir_all("/mnt/etc").unwrap();
    if is_arch() {
        init_keyrings_mirrors(); // Need to initialize keyrings before installing base package group otherwise get keyring errors. It uses rate-mirrors for Arch and Chaotic AUR on the host
        files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf"); // It must be done before installing any Athena and Chaotic AUR package
    }

    let exit_code = match distro_base() {
        Base::AthenaArch   => {
            let code = install(PackageManager::Pacstrap, packages, None);
            files::copy_file("/etc/pacman.d/mirrorlist", "/mnt/etc/pacman.d/mirrorlist"); // It must run after "pacman-mirrorlist" pkg install, that is in base package group
            files::copy_file("/etc/pacman.d/blackarch-mirrorlist", "/mnt/etc/pacman.d/blackarch-mirrorlist");
            files::copy_file("/etc/pacman.d/chaotic-mirrorlist", "/mnt/etc/pacman.d/chaotic-mirrorlist");
            files::copy_file("/mnt/usr/local/share/athena/release/os-release-athena", "/mnt/usr/lib/os-release");
            hardware::set_cores();
            exec_eval(
                exec( // Using exec instead of exec_archchroot because in exec_archchroot, these sed arguments need some chars to be escaped
                    "sed",
                    vec![
                        String::from("-i"),
                        String::from("-e"),
                        String::from("s/^HOOKS=.*/HOOKS=(base systemd autodetect modconf kms keyboard sd-vconsole block sd-encrypt lvm2 filesystems fsck)/g"),
                        String::from("/mnt/etc/mkinitcpio.conf"),
                    ],
                ),
                "Set mkinitcpio hooks",
            );
            files_eval(
                files::sed_file(
                    "/mnt/etc/mkinitcpio.conf",
                    "#COMPRESSION=\"lz4\"",
                    "COMPRESSION=\"gzip\"", // systemd-stub (and therefore UKI) expects an initrd compressed with gzip
                ),
                "Set compression algorithm",
            );
            code
        }

        Base::AthenaFedora => {
            install(PackageManager::Dnf, packages, Some(InstallMode::Install))
        }
        _ => {
            info!("No installation process for selected base system.");
            0
        }
    };

    // Enable the necessary services after installation
    for service in virt_services {
        enable_service(service);
    }

    // After the packages are installed, apply sed commands for virt service
    for (description, args) in virt_params {
        exec_eval(
            exec("sed", args),  // Apply each file change via `sed`
            &description,       // Log the description of the file change
        );
    }
    
    files::copy_file("/etc/skel/.bashrc", "/mnt/etc/skel/.bashrc");

    files_eval(
        files::sed_file(
            "/mnt/etc/nsswitch.conf",
            "hosts:.*",
            "hosts: mymachines resolve [!UNAVAIL=return] files dns mdns wins myhostname",
        ),
        "Set nsswitch configuration",
    );
    exit_code
}

pub fn preset_process() {
    // mkinitcpio -P must be run after all the edits on /etc/mkinitcpio.conf file
    exec_eval(
        exec_archchroot(
            "mkinitcpio",
            vec![
                String::from("-P"),
            ],
        ),
        "Run mkinitcpio presets processing",
    );
}

fn generate_kernel_cmdline() -> String {
    let (fstype, fs_uuid) = detect_root_fs_info();
    let is_btrfs_root = fstype == "btrfs";

    let (luks_partitions, encrypt_check) = find_luks_partitions();

    let mut early_root_param = String::new();

    if encrypt_check {
        // Encrypted root case
        if let Some((device_path, uuid)) = luks_partitions.first() {
            let cryptlabel = format!("{}crypted", device_path.trim_start_matches("/dev/"));
            early_root_param.push_str(&format!("rd.luks.name={uuid}={cryptlabel} "));
            early_root_param.push_str(&format!("root=/dev/mapper/{cryptlabel} "));
        } else {
            error!("encrypt_check=true but luks_partitions is empty");
        }
    } else {
        // Unencrypted root case
        if !fs_uuid.is_empty() {
            early_root_param.push_str(&format!("root=UUID={fs_uuid} "));
        } else {
            error!("Could not determine root UUID for unencrypted root");
        }
    }

    if is_btrfs_root {
        early_root_param.push_str("rootflags=subvol=@ ");
    }

    let mut params: Vec<&str> = vec![
        "lsm=landlock,lockdown,yama,integrity,apparmor,bpf",
        "quiet",
        "loglevel=3",
        "nvme_load=yes",
        "zswap.enabled=0",
        "fbcon=nodefer",
        "nowatchdog",
    ];

    if hardware::is_hyperv_guest() {
        params.push("video=hyperv_fb:3840x2160");
    }

    format!("{early_root_param}{}", params.join(" "))
}

/// Optional: write cmdline to /etc/kernel/cmdline inside target system,
/// so future kernel/UKI regen tools know what to embed.
fn write_kernel_cmdline_file(cmdline: &str) {
    files_eval(files::create_directory("/mnt/etc/kernel"), "Create /mnt/etc/kernel");
    files::create_file("/mnt/etc/kernel/cmdline");
    files_eval(
        files::append_file("/mnt/etc/kernel/cmdline", cmdline),
        "Write /etc/kernel/cmdline",
    );
}

/// Build and sign a Unified Kernel Image (UKI) for one kernel flavor
/// using Arch's ukify syntax.
///
/// kname: "linux-lts" or "linux-hardened"
/// pretty: "LTS" or "Hardened" (for boot menu entry title)
/// esp_str: ESP mount path *inside chroot* (usually "/boot/efi")
/// secureboot_key_dir: path *inside chroot* to the key dir ("/etc/secureboot/keys")
fn build_and_sign_uki(
    kname: &str,
    pretty: &str,
    esp_str: &str,
    secureboot_key_dir: &str,
    cmdline: &str,
) {
    let uki_out = format!("{esp_str}/EFI/Athena/{kname}.efi");

    // ensure ESP/EFI/Athena exists on target fs
    let athena_efi_dir = format!("/mnt{esp_str}/EFI/Athena");
    fs::create_dir_all(&athena_efi_dir)
        .expect("Failed to create /mnt<esp>/EFI/Athena");

    let cpu = hardware::cpu_detect();

    let mut args: Vec<String> = Vec::new();
    args.push("build".into());

    args.push("--linux".into());
    args.push(format!("/boot/vmlinuz-{kname}"));

    // microcode first, conditionally
    if cpu.contains("Intel") {
        args.push("--initrd".into());
        args.push("/boot/intel-ucode.img".into());
    } else if cpu.contains("AMD") {
        args.push("--initrd".into());
        args.push("/boot/amd-ucode.img".into());
    }

    // normal initramfs
    args.push("--initrd".into());
    args.push(format!("/boot/initramfs-{kname}.img"));

    args.push("--cmdline".into());
    args.push(cmdline.to_string());

    args.push("--os-release".into());
    args.push("/usr/lib/os-release".into());

    args.push("--uname".into());
    args.push(kname.to_string());

    args.push("--signtool=sbsign".into());

    args.push("--secureboot-private-key".into());
    args.push(format!("{secureboot_key_dir}/MOK.key"));

    args.push("--secureboot-certificate".into());
    args.push(format!("{secureboot_key_dir}/MOK.crt"));

    args.push("--output".into());
    args.push(uki_out.clone());

    exec_eval(
        exec_archchroot("ukify", args),
        &format!("Create+sign UKI for {kname}"),
    );

    let entries_dir = format!("/mnt{esp_str}/loader/entries");
    fs::create_dir_all(&entries_dir)
        .expect("Failed to create loader/entries dir");
    let entry_path = format!("{entries_dir}/athena-{kname}.conf");

    files::create_file(&entry_path);
    exec_eval(
        files::append_file(
            &entry_path,
            &format!(
                "title   Athena OS ({pretty})\nefi     /EFI/Athena/{kname}.efi\n",
            ),
        ),
        &format!("Write systemd-boot entry for {kname}"),
    );

    info!("UKI for {kname} created at {uki_out} and loader entry {entry_path}");
}

pub fn configure_bootloader_systemd_boot_shim(espdir: PathBuf) {
    let esp_str = espdir.to_str().unwrap();
    info!("Configuring systemd-boot + UKI + shim Secure Boot in {esp_str}");

    // 0. Generate the cmdline ONCE
    let cmdline = generate_kernel_cmdline();

    //    Persist it for future kernel regen
    write_kernel_cmdline_file(&cmdline);

    // 1. Install systemd-boot into ESP
    exec_eval(
        exec_archchroot(
            "bootctl",
            vec![
                "--esp-path".into(),
                esp_str.to_string(),
                "--boot-path".into(), // to create it Linux folder in /boot/efi/EFI instead of /boot/EFI when /boot and /boot/efi mountpoints exist simultaneously
                esp_str.to_string(),
                "install".into(),
            ],
        ),
        "Install systemd-boot",
    );

    // 2. Generate Secure Boot keypair (MOK.key / MOK.crt / MOK.cer)
    let secureboot_key_dir = "/etc/secureboot/keys";
    std::fs::create_dir_all(format!("/mnt{secureboot_key_dir}"))
        .expect("Failed to create secureboot key dir");

    exec_eval(
        exec_archchroot(
            "openssl",
            vec![
                "req".into(),
                "-newkey".into(), "rsa:2048".into(),
                "-nodes".into(),
                "-keyout".into(), format!("{secureboot_key_dir}/MOK.key"),
                "-new".into(),
                "-x509".into(),
                "-sha256".into(),
                "-days".into(), "3650".into(),
                "-subj".into(), "/CN=Athena OS Secure Boot Key/".into(),
                "-out".into(), format!("{secureboot_key_dir}/MOK.crt"),
            ],
        ),
        "Generate Athena Secure Boot keypair",
    );

    exec_eval(
        exec_archchroot(
            "chmod",
            vec![
                "400".into(),
                format!("{secureboot_key_dir}/MOK.key"),
            ],
        ),
        "Restrict Secure Boot private key permissions",
    );

    exec_eval(
        exec_archchroot(
            "openssl",
            vec![
                "x509".into(),
                "-outform".into(), "DER".into(),
                "-in".into(),  format!("{secureboot_key_dir}/MOK.crt"),
                "-out".into(), format!("{secureboot_key_dir}/MOK.cer"),
            ],
        ),
        "Generate DER (.cer) version of Athena Secure Boot cert",
    );

    // 3. Sign systemd-boot itself
    exec_eval(
        exec_archchroot(
            "sbsign",
            vec![
                "--key".into(),  format!("{secureboot_key_dir}/MOK.key"),
                "--cert".into(), format!("{secureboot_key_dir}/MOK.crt"),
                "--output".into(), format!("{esp_str}/EFI/systemd/systemd-bootx64.efi"),
                format!("{esp_str}/EFI/systemd/systemd-bootx64.efi"),
            ],
        ),
        "Sign systemd-boot with Athena key",
    );

    // 4. Build + sign UKIs for BOTH kernels using the SAME cmdline string
    build_and_sign_uki("linux-lts", "LTS", esp_str, secureboot_key_dir, &cmdline);
    build_and_sign_uki("linux-hardened", "Hardened", esp_str, secureboot_key_dir, &cmdline);

    // 5. Write loader.conf, pick linux-lts as default
    let loader_dir = format!("/mnt{esp_str}/loader");
    let entries_dir = format!("{loader_dir}/entries");
    std::fs::create_dir_all(&entries_dir).expect("Failed to create loader/entries dir");

    files::create_file(&format!("{loader_dir}/loader.conf"));
    files_eval(
        files::append_file(
            &format!("{loader_dir}/loader.conf"),
            "default athena-linux-lts.conf\ntimeout 3\nconsole-mode keep\neditor no\n",
        ),
        "Write loader.conf",
    );

    // 6. Set up shim as stage0 so Secure Boot works out-of-the-box,
    //    and so first boot triggers MOK Manager instead of forcing firmware DB enrollment.
    //
    // Firmware will execute BOOTX64.EFI. We want that to be shim (Microsoft-signed),
    // so Secure Boot allows it immediately.
    //
    // Then shim will try to load "grubx64.efi". We give it *our signed systemd-boot*
    // under that name, and after the user enrolls AthenaSecureBoot.cer in MOK Manager,
    // shim will allow it.
    //
    // So:
    //   ESP/EFI/BOOT/BOOTX64.EFI      <- shimx64.efi (MS-signed, from shim-signed pkg)
    //   ESP/EFI/BOOT/grubx64.efi      <- signed systemd-bootx64.efi
    //
    std::fs::create_dir_all(format!("/mnt{esp_str}/EFI/BOOT"))
        .expect("Failed to create ESP/EFI/BOOT directory");

    // Copy shim
    files::copy_file(
        "/mnt/usr/share/shim-signed/shimx64.efi",
        &format!("/mnt{esp_str}/EFI/BOOT/BOOTX64.EFI"),
    );

    // Copy MokManager so shim can start MOK enrollment UI at first boot
    files::copy_file(
        "/mnt/usr/share/shim-signed/mmx64.efi",
        &format!("/mnt{esp_str}/EFI/BOOT/mmx64.efi"),
    );

    // Copy our signed systemd-boot binary where shim expects "grubx64.efi"
    files::copy_file(
        &format!("/mnt{esp_str}/EFI/systemd/systemd-bootx64.efi"),
        &format!("/mnt{esp_str}/EFI/BOOT/grubx64.efi"),
    );

    // 7. Copy Athena public cert somewhere obvious on ESP.
    //    Shim/MOK Manager will ask to enroll it on first boot (after mokutil below).
    std::fs::create_dir_all(format!("/mnt{esp_str}/EFI/Athena"))
        .expect("Failed to create ESP/EFI/Athena directory");
    files::copy_file(
        &format!("/mnt{secureboot_key_dir}/MOK.cer"),
        &format!("/mnt{esp_str}/EFI/Athena/AthenaSecureBoot.cer"),
    );

    // 8. Pre-register the Athena key with mokutil so first boot asks user
    //    "Enroll this key?" in MOK Manager. This avoids making them open BIOS UI.
    exec_eval(
        exec_archchroot(
            "mokutil",
            vec![
                "--import".into(),
                format!("{secureboot_key_dir}/MOK.cer"),
                "-P".into(), // no password prompt path. If you prefer pwd-confirm flow,
                             // remove -P and handle mokutil --password instead.
            ],
        ),
        "Schedule AthenaSecureBoot.cer enrollment in MOK Manager at first boot",
    );

    info!("systemd-boot + UKI + shim configured. On first boot, MOK Manager will ask to enroll AthenaSecureBoot.cer; accept it to boot securely without touching firmware setup.");
}

fn init_keyrings_mirrors() {
    info!("Getting fastest mirrors for your location");
    exec_eval(
        exec( // It is done on the live system
            "rate-mirrors",
            vec![
                String::from("--concurrency"),
                String::from("40"),
                String::from("--disable-comments"),
                String::from("--allow-root"),
                String::from("--save"),
                String::from("/etc/pacman.d/mirrorlist"), // It must be saved not in the chroot environment but on the host machine of Live Environment. Next, it will be copied automatically on the target system.
                String::from("arch"),
            ],
        ),
        "Set fastest Arch Linux mirrors on the host",
    );

    exec_eval(
        exec(
            "rate-mirrors",
            vec![
                String::from("--concurrency"),
                String::from("40"),
                String::from("--disable-comments"),
                String::from("--allow-root"),
                String::from("--save"),
                String::from("/etc/pacman.d/blackarch-mirrorlist"),
                String::from("blackarch"),
            ],
        ),
        "Set fastest mirrors from BlackArch on the target system",
    );
    
    exec_eval(
        exec(
            "rate-mirrors",
            vec![
                String::from("--concurrency"),
                String::from("40"),
                String::from("--disable-comments"),
                String::from("--allow-root"),
                String::from("--save"),
                String::from("/etc/pacman.d/chaotic-mirrorlist"), //In chroot we don't need to specify /mnt
                String::from("chaotic-aur"),
            ],
        ),
        "Set fastest mirrors from Chaotic AUR on the target system",
    );
    
    info!("Upgrade keyrings on the host");
    exec_eval(
        exec(
            "rm",
            vec![
                String::from("-rf"),
                String::from("/etc/pacman.d/gnupg"),
            ],
        ),
        "Removing keys",
    );
    exec_eval(
        exec(
            "pacman-key",
            vec![
                String::from("--init"),
            ],
        ),
        "Initialize keys",
    );
    exec_eval(
        exec(
            "pacman-key",
            vec![
                String::from("--populate"),
            ],
        ),
        "Populate keys",
    );
}

pub fn genfstab() {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                String::from("genfstab -U /mnt >> /mnt/etc/fstab"),
            ],
        ),
        "Generate fstab",
    );
}

pub fn install_nix_config() {
    // Increase the rootfs size
    exec_eval(
        exec(
            "mount",
            vec![
                String::from("-o"),
                String::from("remount,size=4G"),
                String::from("/run"),
            ],
        ),
        "Increase the rootfs partition size",
    );
    info!("Set nix channels.");
    // As channel we use nixos-unstable instead of nixpkgs-unstable because 'nixos-' has additional tests that ensure kernel and bootloaders actually work. And some other critical packages.
    exec_eval(
        exec(
            "nix-channel",
            vec![
                String::from("--add"),
                String::from("https://nixos.org/channels/nixos-unstable"),
                String::from("nixpkgs"),
            ],
        ),
        "Set nixpkgs nix channel on the host",
    );
    // This update is done on the host, not on the target system
    exec_eval(
        exec(
            "nix-channel",
            vec![
                String::from("--update"),
            ],
        ),
        "Update nix channels on the host",
    );
    std::fs::create_dir_all("/mnt/etc/nixos").unwrap();
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

fn detect_root_fs_info() -> (String, String) {
    // Ask findmnt for both filesystem type and UUID of /mnt in one go.
    let output = exec_eval_result(
        exec_output(
            "findmnt",
            vec![
                "-n".into(),
                "-o".into(),
                "FSTYPE,UUID".into(),
                "/mnt".into(),
            ],
        ),
        "Detect filesystem type and UUID for /mnt",
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split_whitespace().collect();

    let fstype = parts.first().unwrap_or(&"").to_string();
    let uuid = parts.get(1).unwrap_or(&"").to_string();

    if fstype.is_empty() {
        error!("Failed to detect filesystem type for /mnt");
    }

    if uuid.is_empty() {
        error!("Failed to detect filesystem UUID for /mnt");
    }

    (fstype, uuid)
}

pub fn configure_zram() {
    files::create_file("/mnt/etc/systemd/zram-generator.conf");
    files_eval(
        files::append_file("/mnt/etc/systemd/zram-generator.conf", "[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd\nswap-priority = 100\nfs-type = swap"),
        "Write zram-generator config",
    );
}

pub fn enable_system_services() {
    enable_service("apparmor");
    enable_service("auditd");
    enable_service("bluetooth");
    enable_service("irqbalance");
    enable_service("NetworkManager");
    enable_service("podman");
    enable_service("vnstat");
    if is_arch() {
        enable_service("ananicy");
        enable_service("cronie");
        enable_service("systemd-timesyncd");
    }
}
