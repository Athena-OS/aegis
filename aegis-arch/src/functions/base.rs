use crate::internal::hardware;
use crate::internal::install::install;
use crate::internal::services::enable_service;
use shared::args::PackageManager;
use shared::exec::exec;
use shared::exec::exec_archchroot;
use shared::encrypt::find_luks_partitions;
use shared::files;
use shared::{info, warn};
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;
use shared::strings::crash;
use std::path::PathBuf;

pub fn install_packages(kernel: String, mut packages: Vec<&str>) {

    let (kernel_to_install, kernel_headers_to_install) = if kernel.is_empty() {
        ("linux-lts", "linux-lts-headers")
    } else {
        match kernel.as_str() {
            "linux" => ("linux", "linux-headers"),
            "linux lts" => ("linux-lts", "linux-lts-headers"),
            "linux zen" => ("linux-zen", "linux-zen-headers"),
            "linux hardened" => ("linux-hardened", "linux-hardened-headers"),
            "linux real-time" => ("linux-rt", "linux-rt-headers"),
            "linux real-time lts" => ("linux-rt-lts", "linux-rt-lts-headers"),
            "linux liquorix" => ("linux-lqx", "linux-lqx-headers"),
            "linux xanmod" => ("linux-xanmod", "linux-xanmod-headers"),
            _ => {
                warn!("Unknown kernel: {}, using default instead", kernel);
                ("linux-lts", "linux-lts-headers")
            }
        }
    };
    let mut base_packages: Vec<&str> = vec![
        // Kernel
        kernel_to_install,
        kernel_headers_to_install,
        "linux-firmware",
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

    // Add multiple strings from another Vec
    packages.append(&mut base_packages);

    std::fs::create_dir_all("/mnt/etc").unwrap();
    init_keyrings_mirrors(); // Need to initialize keyrings before installing base package group otherwise get keyring errors. It uses rate-mirrors for Arch and Chaotic AUR on the host
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf"); // It must be done before installing any Athena and Chaotic AUR package

    let (virt_packages, virt_services, virt_params) = hardware::virt_check();
    let gpu_packages = hardware::cpu_gpu_check(kernel_to_install);
    packages.extend(virt_packages);
    packages.extend(gpu_packages);

    // These packages are installed by Pacstrap, so by using host mirrors
    install(PackageManager::Pacstrap, packages);

    files::copy_file("/etc/pacman.d/mirrorlist", "/mnt/etc/pacman.d/mirrorlist"); // It must run after "pacman-mirrorlist" pkg install, that is in base package group
    files::copy_file("/etc/pacman.d/chaotic-mirrorlist", "/mnt/etc/pacman.d/chaotic-mirrorlist");

    hardware::set_cores();

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
    
    files::copy_file("/etc/skel/.bashrc", "/mnt/etc/skel/.bashrc");
    files::copy_file("/mnt/usr/local/share/athena/release/os-release-athena", "/mnt/usr/lib/os-release");
    files::copy_file("/etc/grub.d/40_custom", "/mnt/etc/grub.d/40_custom");

    files_eval(
        files::sed_file(
            "/mnt/etc/mkinitcpio.conf",
            "#COMPRESSION=\"lz4\"",
            "COMPRESSION=\"lz4\"",
        ),
        "Set compression algorithm",
    );

    files_eval(
        files::sed_file(
            "/mnt/etc/nsswitch.conf",
            "hosts:.*",
            "hosts: mymachines resolve [!UNAVAIL=return] files dns mdns wins myhostname",
        ),
        "Set nsswitch configuration",
    );
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
    info!("Getting fastest Arch and Chaotic AUR mirrors for your location");
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
                String::from("/etc/pacman.d/chaotic-mirrorlist"), //In chroot we don't need to specify /mnt
                String::from("chaotic-aur"),
            ],
        ),
        "Set fastest mirrors from Chaotic AUR on the target system",
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

fn setting_grub_parameters(encrypt_check: bool) {
    let mut luks_param = String::new();
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_DISTRIBUTOR=.*",
            "GRUB_DISTRIBUTOR=\"Athena OS\"",
        ),
        "Set distributor name",
    );
    if encrypt_check {
        /*Set UUID of encrypted partition as kernel parameter*/
        let luks_partitions = find_luks_partitions();
        let mut cryptlabel = String::new();
        info!("LUKS partitions found:");
        for (device_path, uuid) in &luks_partitions {
            info!("Device: {}, UUID: {}", device_path, uuid);
            cryptlabel = format!("{}crypted", device_path.trim_start_matches("/dev/")); // i.e., sda3crypted
            luks_param.push_str(&format!("rd.luks.name={}={} ", uuid, cryptlabel));
        }
        luks_param.push_str(&format!("root=/dev/mapper/{} ", cryptlabel));
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
            &format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}quiet loglevel=3 audit=0 nvme_load=yes zswap.enabled=0 fbcon=nodefer nowatchdog\"", luks_param),
        ),
        "Set kernel parameters",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "#GRUB_DISABLE_OS_PROBER=.*",
            "GRUB_DISABLE_OS_PROBER=false",
        ),
        "Enable os prober",
    );
}

pub fn configure_bootloader_efi(efidir: PathBuf, encrypt_check: bool) {

    let efidir = std::path::Path::new("/mnt").join(&efidir);
    let efi_str = efidir.to_str().unwrap();
    info!("EFI bootloader installing at {}", efi_str);
    
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    
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

    setting_grub_parameters(encrypt_check);
    
    exec_eval(
        exec_archchroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "Create grub.cfg",
    );
}

pub fn configure_bootloader_legacy(device: PathBuf, encrypt_check: bool) {

    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }

    let device_str = device.to_string_lossy().to_string();
    info!("Legacy bootloader installing at {}", device_str);

    exec_eval(
        exec_archchroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device_str],
        ),
        "Install grub as legacy",
    );

    setting_grub_parameters(encrypt_check);
    
    exec_eval(
        exec_archchroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "Create grub.cfg",
    );
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

pub fn configure_flatpak() {
    exec_eval(
        exec_archchroot(
            "flatpak",
            vec![
                String::from("remote-add"),
                String::from("--if-not-exists"),
                String::from("flathub"),
                String::from("https://flathub.org/repo/flathub.flatpakrepo"),
            ],
        ),
        "Add flathub remote",
    )
}

pub fn configure_zram() {
    files::create_file("/mnt/etc/systemd/zram-generator.conf");
    files_eval(
        files::append_file("/mnt/etc/systemd/zram-generator.conf", "[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd\nswap-priority = 100\nfs-type = swap"),
        "Write zram-generator config",
    );
}

pub fn enable_system_services() {
    enable_service("ananicy");
    enable_service("auditd");
    enable_service("bluetooth");
    enable_service("cronie");
    enable_service("irqbalance");
    enable_service("NetworkManager");
    enable_service("set-cfs-tweaks");
    enable_service("systemd-timesyncd");
    enable_service("vnstat");
    //enable_service("nohang");
    //enable_service("cups");
}
