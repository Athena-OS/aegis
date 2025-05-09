use crate::internal::hardware;
use crate::internal::install::install;
use crate::internal::services;
use shared::args::InstallMode;
use shared::args::PackageManager;
use shared::exec::exec;
use shared::exec::exec_chroot;
use shared::exec::exec_output;
use shared::encrypt::find_luks_partitions;
use shared::files;
use shared::info;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::exec_eval_result;
use shared::returncode_eval::files_eval;
use shared::strings::crash;
use std::path::PathBuf;

pub fn install_packages(mut packages: Vec<&str>) {

    let mut base_packages: Vec<&str> = vec![
        // Kernel
        "kernel",
        "kernel-modules",
        "kernel-modules-extra",
        "kernel-headers",
        "linux-firmware",
        "glibc-all-langpacks", // Prebuilt locales
        "https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm",
        "https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm",
        ];

/*
    let pre_packages: Vec<&str> = vec![
        "gmp",
        "coreutils",
    ];
    install(PackageManager::Dnf, pre_packages, InstallMode::Install);
*/

    // Add multiple strings from another Vec
    packages.append(&mut base_packages);

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
        packages.extend(["btrfs-progs"]);
    }
    info!("Root partition is {}", fstype);

    std::fs::create_dir_all("/mnt/etc/yum.repos.d").unwrap();
    files::copy_multiple_files("/etc/yum.repos.d/*", "/mnt/etc/yum.repos.d");
    std::fs::create_dir_all("/mnt/etc/default").unwrap();
    files::copy_file("/etc/default/grub", "/mnt/etc/default/grub");

    let (virt_packages, virt_services, virt_params) = hardware::virt_check();
    let gpu_packages = hardware::cpu_gpu_check();
    packages.extend(virt_packages);
    packages.extend(gpu_packages);

    // These packages are installed by Dnf, so by using host repositories
    install(PackageManager::Dnf, packages, InstallMode::Install);
    //install(PackageManager::Dnf, rm_packages, InstallMode::Remove);



    // Enable the necessary services after installation
    for service in virt_services {
        services::enable_service(service);
    }

    // After the packages are installed, apply sed commands for virt service
    for (description, args) in virt_params {
        exec_eval(
            exec("sed", args),  // Apply each file change via `sed`
            &description,       // Log the description of the file change
        );
    }
    
    files::copy_file("/home/liveuser/.bashrc", "/mnt/etc/skel/.bashrc");
    files::copy_file("/etc/grub.d/40_custom", "/mnt/etc/grub.d/40_custom");

    files_eval(
        files::sed_file(
            "/mnt/etc/nsswitch.conf",
            "hosts:.*",
            "hosts: mymachines resolve [!UNAVAIL=return] files dns mdns wins myhostname",
        ),
        "Set nsswitch configuration",
    );
}

pub fn genfstab() {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                String::from("genfstab -U -S /mnt >> /mnt/etc/fstab"),
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
        "set distributor name",
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
            "set grub encrypt parameter",
        );
    }
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_CMDLINE_LINUX_DEFAULT=.*",
            &format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}quiet loglevel=3 nvme_load=yes zswap.enabled=0 fbcon=nodefer nowatchdog\"", luks_param),
        ),
        "set kernel parameters",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "#GRUB_DISABLE_OS_PROBER=.*",
            "GRUB_DISABLE_OS_PROBER=false",
        ),
        "enable os prober",
    );
}

pub fn configure_bootloader_efi(efidir: PathBuf, encrypt_check: bool) {

    let efidir = std::path::Path::new("/mnt").join(&efidir);
    let efi_str = efidir.to_str().unwrap();
    info!("EFI bootloader installing at {}", efi_str);
    
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }

    setting_grub_parameters(encrypt_check);
    
    exec_eval(
        exec_chroot(
            "grub2-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub2/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn configure_bootloader_legacy(device: PathBuf, encrypt_check: bool) {

    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }

    let device_str = device.to_string_lossy().to_string();
    info!("Legacy bootloader installing at {}", device_str);

    exec_eval(
        exec_chroot(
            "grub2-install",
            vec![String::from("--target=i386-pc"), device_str],
        ),
        "install grub as legacy",
    );

    setting_grub_parameters(encrypt_check);
    
    exec_eval(
        exec_chroot(
            "grub2-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub2/grub.cfg")],
        ),
        "create grub.cfg",
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
        exec_chroot(
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
        exec_chroot(
            "flatpak",
            vec![
                String::from("remote-add"),
                String::from("--if-not-exists"),
                String::from("flathub"),
                String::from("https://flathub.org/repo/flathub.flatpakrepo"),
            ],
        ),
        "add flathub remote",
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
    services::enable_service("auditd");
    services::enable_service("bluetooth");
    services::enable_service("crond");
    services::enable_service("irqbalance");
    services::enable_service("NetworkManager");
    services::enable_service("vnstat");
    //services::enable_service("nohang");
    //services::enable_service("cups");
}
