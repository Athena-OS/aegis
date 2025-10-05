use crate::internal::hardware;
use crate::internal::install::install;
use crate::internal::services::enable_service;
use log::info;
use shared::args::{Base, distro_base, ExtendIntoString, InstallMode, PackageManager, is_arch, is_fedora, is_nix};
use shared::exec::{exec, exec_archchroot, exec_output};
use shared::encrypt::find_luks_partitions;
use shared::files;
use shared::returncode_eval::{exec_eval, exec_eval_result, files_eval};
use shared::strings::crash;
use std::path::PathBuf;

pub fn install_packages(mut packages: Vec<String>) -> i32 {
    let (kernel_to_install, kernel_headers_to_install) = ("linux-lts", "linux-lts-headers");
    let arch_base_pkg: Vec<&str> = vec![
        // Kernel
        kernel_to_install,
        kernel_headers_to_install,
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

    let fedora_base_pkg: Vec<&str> = vec![
        "kernel",
        "kernel-modules",
        "kernel-modules-extra",
        "kernel-headers",
        "glibc-all-langpacks",
    ];

    if is_arch() {
        packages.extend_into(arch_base_pkg);
    } else if is_fedora() {
        packages.extend_into(fedora_base_pkg);
    }

    /***** CHECK IF BTRFS *****/
    let output = exec_eval_result(
        exec_output(
            "findmnt",
            vec![
                String::from("-n"),
                String::from("-o"),
                String::from("FSTYPE"),
                String::from("/mnt"),
            ],
        ),
        "Detect file system type",
    );

    let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
    } else if is_fedora() {
        std::fs::create_dir_all("/mnt/etc/yum.repos.d").unwrap();
        files::copy_multiple_files("/etc/yum.repos.d/*", "/mnt/etc/yum.repos.d");
        std::fs::create_dir_all("/mnt/etc/default").unwrap();
        files::copy_file("/etc/default/grub", "/mnt/etc/default/grub");        
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
                    "COMPRESSION=\"lz4\"",
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
    files::copy_file("/etc/grub.d/40_custom", "/mnt/etc/grub.d/40_custom");

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
                String::from("genfstab-fedora -U /mnt >> /mnt/etc/fstab"),
            ],
        ),
        "Generate fstab",
    );
}

fn setting_grub_parameters() {
    let mut luks_param = String::new();
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_DISTRIBUTOR=.*",
            "GRUB_DISTRIBUTOR=\"Athena OS\"",
        ),
        "Set distributor name",
    );
    let (luks_partitions, encrypt_check) = find_luks_partitions();
    if encrypt_check {
        /*Set UUID of encrypted partition as kernel parameter*/
        let mut cryptlabel = String::new();
        info!("LUKS partitions found:");
        for (device_path, uuid) in &luks_partitions {
            info!("Device: {device_path}, UUID: {uuid}");
            cryptlabel = format!("{}crypted", device_path.trim_start_matches("/dev/")); // i.e., sda3crypted
            luks_param.push_str(&format!("rd.luks.name={uuid}={cryptlabel} "));
        }
        luks_param.push_str(&format!("root=/dev/mapper/{cryptlabel} "));
        // NOTE: in case of multiple LUKS encryted partitions, the encrypted system will work ONLY if the root partition is the last one in the disk

        files_eval(
            files::sed_file(
                "/mnt/etc/default/grub",
                "#GRUB_ENABLE_CRYPTODISK=.*",
                "GRUB_ENABLE_CRYPTODISK=y",
            ),
            "Set grub encrypt parameter",
        );
    }
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_CMDLINE_LINUX_DEFAULT=.*",
            &format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{luks_param}quiet loglevel=3 nvme_load=yes zswap.enabled=0 fbcon=nodefer nowatchdog\""),
        ),
        "Set kernel parameters",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "#GRUB_DISABLE_OS_PROBER=.*",
            "GRUB_DISABLE_OS_PROBER=false",
        ),
        "Enable OS prober",
    );
}

pub fn configure_bootloader_efi(efidir: PathBuf) {

    let efidir = std::path::Path::new("/mnt").join(&efidir);
    let efi_str = efidir.to_str().unwrap();
    info!("EFI bootloader installing at {efi_str}");
    
    if !std::path::Path::new(efi_str).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    
    if is_arch() {
        exec_eval(
            exec_archchroot(
                "grub-install",
                vec![
                    String::from("--target=x86_64-efi"),
                    format!("--efi-directory={}", efi_str),
                    String::from("--bootloader-id=GRUB"),
                    String::from("--removable"),
                ],
            ),
            "Install grub as efi with --removable",
        );

        exec_eval(
            exec_archchroot(
                "grub-install",
                vec![
                    String::from("--target=x86_64-efi"),
                    format!("--efi-directory={}", efi_str),
                    String::from("--bootloader-id=GRUB"),
                ],
            ),
            "Install grub as efi without --removable",
        );
        setting_grub_parameters();
        exec_eval(
            exec_archchroot(
                "grub-mkconfig",
                vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
            ),
            "Create grub.cfg",
        );
    }

    
    
    if is_fedora() {
        setting_grub_parameters();
        exec_eval(
            exec_archchroot(
                "grub2-mkconfig",
                vec![String::from("-o"), String::from("/boot/grub2/grub.cfg")],
            ),
            "Create grub.cfg",
        );
    }
}

pub fn configure_bootloader_legacy(device: PathBuf) {

    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }

    let device_str = device.to_string_lossy().to_string();
    info!("Legacy bootloader installing at {device_str}");

    if is_nix () {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/modules/boot/grub/default.nix",
                "/dev/sda",
                &device_str,
            ),
            "Set Legacy bootloader device",
        );
    }

    if is_arch() {
        exec_eval(
            exec_archchroot(
                "grub-install",
                vec![String::from("--target=i386-pc"), device_str],
            ),
            "Install GRUB as legacy",
        );
        setting_grub_parameters();
        exec_eval(
            exec_archchroot(
                "grub-mkconfig",
                vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
            ),
            "Create grub.cfg",
        );
    } else if is_fedora() {
        exec_eval(
            exec_archchroot(
                "grub2-install",
                vec![String::from("--target=i386-pc"), device_str],
            ),
            "Install GRUB as legacy",
        );
        setting_grub_parameters();
        exec_eval(
            exec_archchroot(
                "grub2-mkconfig",
                vec![String::from("-o"), String::from("/boot/grub2/grub.cfg")],
            ),
            "Create grub.cfg",
        );
    }
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

/*
pub fn setup_snapper() {
    install(PackageManager::Pacman, vec![
        "btrfs-assistant", "btrfs-progs", "btrfsmaintenance", "grub-btrfs", "inotify-tools", "snap-pac", "snap-pac-grub", "snapper-support",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub-btrfs/config",
            "#GRUB_BTRFS_LIMIT=.*",
            "GRUB_BTRFS_LIMIT=\"5\"",
        ),
        "Set Grub Btrfs limit",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub-btrfs/config",
            "#GRUB_BTRFS_SHOW_SNAPSHOTS_FOUND=.*",
            "GRUB_BTRFS_SHOW_SNAPSHOTS_FOUND=\"false\"",
        ),
        "Not show Grub Btrfs snapshots found",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub-btrfs/config",
            "#GRUB_BTRFS_SHOW_TOTAL_SNAPSHOTS_FOUND=.*",
            "GRUB_BTRFS_SHOW_TOTAL_SNAPSHOTS_FOUND=\"false\"",
        ),
        "Not show the total number of Grub Btrfs snapshots found",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/conf.d/snapper",
            "SNAPPER_CONFIGS=.*",
            "SNAPPER_CONFIGS=\"root\"",
        ),
        "Not show the total number of Grub Btrfs snapshots found",
    );
    exec_eval(
        exec_archchroot(
            "btrfs",
            vec![
                String::from("subvolume"),
                String::from("create"),
                String::from("/.snapshots"),
            ],
        ),
        "create /.snapshots as btrfs subvolume",
    );
    files::copy_file("/mnt/etc/snapper/config-templates/garuda", "/mnt/etc/snapper/configs/root");
    enable_service("grub-btrfsd");
}
*/

pub fn configure_zram() {
    files::create_file("/mnt/etc/systemd/zram-generator.conf");
    files_eval(
        files::append_file("/mnt/etc/systemd/zram-generator.conf", "[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd\nswap-priority = 100\nfs-type = swap"),
        "Write zram-generator config",
    );
}

pub fn enable_system_services() {
    enable_service("auditd");
    enable_service("bluetooth");
    enable_service("irqbalance");
    enable_service("NetworkManager");
    enable_service("podman");
    enable_service("vnstat");
    if is_arch() {
        enable_service("ananicy");
        enable_service("cronie");
        enable_service("set-cfs-tweaks");
        enable_service("systemd-timesyncd");
    } else if is_fedora() {
        enable_service("crond");
    }
}
