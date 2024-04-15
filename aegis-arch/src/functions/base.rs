use crate::internal::hardware;
use crate::internal::install::install;
use crate::internal::services::enable_service;
use shared::args::PackageManager;
use shared::exec::exec;
use shared::exec::exec_chroot;
use shared::encrypt::find_luks_partitions;
use shared::files;
use shared::{info, warn};
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;
use shared::strings::crash;
use std::path::PathBuf;

pub fn install_base_packages() {

    std::fs::create_dir_all("/mnt/etc").unwrap();
    initialize_keyrings(); // Need to initialize keyrings before installing base package group otherwise get keyring errors. It uses rate-mirrors too
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf"); // It must be done before installing any Athena and Chaotic AUR package
    install(PackageManager::Pacstrap, vec![
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
    ]);
    files::copy_file("/etc/pacman.d/mirrorlist", "/mnt/etc/pacman.d/mirrorlist"); // It must run after "pacman-mirrorlist" pkg install, that is in base package group
    fastest_mirrors(); // Done on the target system
}

pub fn install_packages(kernel: String) {

    let kernel_to_install = if kernel.is_empty() {
        "linux-lts"
    } else {
        match kernel.as_str() {
            "linux" => "linux",
            "linux lts" => "linux-lts",
            "linux zen" => "linux-zen",
            "linux hardened" => "linux-hardened",
            "linux real-time" => "linux-rt",
            "linux real-time lts" => "linux-rt-lts",
            "linux liquorix" => "linux-lqx",
            "linux xanmod" => "linux-xanmod",
            _ => {
                warn!("Unknown kernel: {}, using default instead", kernel);
                "linux-lts"
            }
        }
    };

    
    install(PackageManager::Pacman, vec![
        // System Arch
        kernel_to_install,
        format!("{kernel_to_install}-headers").as_str(),
        "linux-firmware",
        "systemd-sysvcompat",
        "networkmanager",
        "network-manager-applet",
        "man-db",
        "man-pages",
        "texinfo",
        "nano",
        "sudo",
        "curl",
        // Extra Base Arch
        "accountsservice",
        "alacritty",
        "alsa-utils",
        "arch-install-scripts",
        "broadcom-wl-dkms",
        "dhcpcd",
        "dialog",
        "dosfstools",
        "edk2-shell",
        "inetutils",
        "irqbalance",
        "lvm2",
        "memtest86+",
        "mesa",
        "mesa-utils",
        "mkinitcpio-nfs-utils",
        "mkinitcpio-openswap",
        "most",
        "mtools",
        "nbd",
        "net-tools",
        "netctl",
        "nfs-utils",
        "nohang",
        "nss-mdns",
        "ntfsprogs",
        "ntp",
        "pavucontrol",
        "profile-sync-daemon",
        "pv",
        "rsync",
        "rtl8821cu-morrownr-dkms-git",
        "sof-firmware",
        "squashfs-tools",
        "syslinux",
        "testdisk",
        "timelineproject-hg",
        "usbutils",
        "wireless_tools",
        "wpa_supplicant",
        "xfsprogs",
        // Fonts
        "noto-fonts",
        "noto-fonts-emoji",
        "noto-fonts-cjk",
        // Common packages for all desktops
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "ntfs-3g",
        "vi",
        "eza",
        "pocl", // Hashcat dependency
        "ananicy",
        "armcord-git",
        "asciinema",
        "bashtop",
        "bat",
        "bc",
        "bless",
        "chatgpt-desktop-bin",
        "cmatrix",
        "cowsay",
        "cron",
        "cyberchef-electron",
        "downgrade",
        "eog",
        "espeakup",
        "figlet",
        "figlet-fonts",
        "file-roller",
        "fortune-mod",
        "git",
        "gparted",
        "grub-customizer",
        "gtk-engine-murrine",
        "gvfs-gphoto2",
        "gvfs-mtp",
        "hexedit",
        //"hw-probe, //HW probing
        "imagemagick",
        "jq",
        "lib32-glibc",
        "lolcat",
        "lsd",
        "mtpfs",
        "nano-syntax-highlighting",
        "nautilus",
        "ncdu",
        "networkmanager-openvpn",
        "nyancat",
        "octopi",
        "onionshare",
        "openbsd-netcat",
        "openvpn",
        "orca",
        "p7zip",
        "paru",
        "pfetch",
        "polkit",
        "python-pywhat",
        "reflector",
        "sl",
        //"smartmontools", //hw-probe deps
        "superbfetch-git",
        "textart",
        "tidy",
        "tk",
        "toilet-fonts",
        "torbrowser-launcher",
        "tree",
        "ufw",
        "unzip",
        "vnstat",
        "wget",
        "which",
        "xclip",
        "xmlstarlet",
        "zoxide",
        // Athena
        "athena-cyber-hub",
        "athena-neofetch-config",
        "athena-nvim-config",
        "athena-powershell-config",
        "athena-config",
        "athena-theme-tweak",
        "athena-tmux-config",
        "athena-vim-config",
        "athena-vscodium-themes",
        "athena-welcome",
        "htb-toolkit",
        "nist-feed",
    ]);

    hardware::set_cores();
    hardware::cpu_gpu_check(kernel_to_install);
    hardware::virt_check();

    exec_eval(
        exec( // Using exec instead of exec_chroot because in exec_chroot, these sed arguments need some chars to be escaped
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
    files::copy_file("/mnt/usr/lib/os-release-athena", "/mnt/usr/lib/os-release");
    files::copy_file("/etc/grub.d/40_custom", "/mnt/etc/grub.d/40_custom");
    // Copy the content of system-connections to the target system in order to keep active WiFi connection even after the installation
    files::copy_all_files("/etc/NetworkManager/system-connections", "/mnt/etc/NetworkManager/system-connections");

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
        exec_chroot(
            "mkinitcpio",
            vec![
                String::from("-P"),
            ],
        ),
        "run mkinitcpio presets processing",
    );
}

fn initialize_keyrings() {
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
        "Set fastest Arch Linux mirrors",
    );
}

fn fastest_mirrors() {
    info!("Getting fastest Chaotic AUR mirrors for your location");
    exec_eval(
        exec_chroot(
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
        "Set fastest mirrors from Chaotic AUR",
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
            &format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}quiet loglevel=3 audit=0 nvme_load=yes zswap.enabled=0 fbcon=nodefer nowatchdog\"", luks_param),
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

pub fn install_bootloader_efi(efidir: PathBuf, encrypt_check: bool) {
    install(PackageManager::Pacman, vec![
        "grub",
        "efibootmgr",
        "os-prober",
        "athena-grub-theme",
    ]);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    info!("EFI bootloader installing at {}", efi_str);
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=GRUB"),
                String::from("--removable"),
            ],
        ),
        "install grub as efi with --removable",
    );
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=GRUB"),
            ],
        ),
        "install grub as efi without --removable",
    );
    setting_grub_parameters(encrypt_check);
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn install_bootloader_legacy(device: PathBuf, encrypt_check: bool) {
    install(PackageManager::Pacman, vec![
        "grub",
        "os-prober",
        "athena-grub-theme",
    ]);
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    info!("Legacy bootloader installing at {}", device);
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device],
        ),
        "install grub as legacy",
    );
    setting_grub_parameters(encrypt_check);
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
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

pub fn install_flatpak() {
    install(PackageManager::Pacman, vec!["flatpak"]);
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

pub fn install_cuda() {
    install(PackageManager::Pacman, vec!["cuda"]);
}

pub fn install_spotify() {
    install(PackageManager::Pacman, vec!["spotify"]);
}

pub fn install_cherrytree() {
    install(PackageManager::Pacman, vec!["cherrytree"]);
}

pub fn install_flameshot() {
    install(PackageManager::Pacman, vec!["flameshot"]);
}

pub fn install_busybox() {
    install(PackageManager::Pacman, vec!["busybox"]);
}

pub fn install_toybox() {
    install(PackageManager::Pacman, vec!["toybox"]);
}

pub fn install_zram() {
    install(PackageManager::Pacman, vec!["zram-generator"]);
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
