use crate::args::PackageManager;
use crate::internal::exec::*;
use crate::internal::services::enable_service;
use crate::internal::*;
use log::warn;
use std::path::PathBuf;

pub fn install_base_packages() {

    std::fs::create_dir_all("/mnt/etc").unwrap();
    initialize_keyrings(); // Need to initialize keyrings before installing base package group otherwise get keyring errors. It uses rate-mirrors too
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf"); // It must be done before installing any Athena, BlackArch and Chaotic AUR package
    install::install(PackageManager::Pacstrap, vec![
        // Base Arch
        "base",
        "glibc-locales", // Prebuilt locales to prevent locales warning message during the pacstrap install of base metapackage
        // Repositories
        "athena-mirrorlist",
        "blackarch-mirrorlist",
        "chaotic-mirrorlist",
        "rate-mirrors",
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

    
    install::install(PackageManager::Pacman, vec![
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
        "edex-ui-bin",
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
        "jdk-openjdk",
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
        "xcp",
        "xmlstarlet",
        // Athena
        "athena-cyber-hub",
        "athena-neofetch-config",
        "athena-nvim-config",
        "athena-powershell-config",
        "athena-system-config",
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
    
    files::copy_file("/etc/skel/.bashrc", "/mnt/etc/skel/.bashrc");
    files::copy_file("/mnt/usr/lib/os-release-athena", "/mnt/usr/lib/os-release");
    files::copy_file("/etc/grub.d/40_custom", "/mnt/etc/grub.d/40_custom");

    files_eval(
        files::sed_file(
            "/mnt/etc/mkinitcpio.conf",
            "#COMPRESSION=\"lz4\"",
            "COMPRESSION=\"lz4\"",
        ),
        "set distributor name",
    );

    files_eval(
        files::sed_file(
            "/mnt/etc/nsswitch.conf",
            "hosts:.*",
            "hosts: mymachines resolve [!UNAVAIL=return] files dns mdns wins myhostname",
        ),
        "set distributor name",
    );
}

fn initialize_keyrings() {
    log::info!("Upgrade keyrings on the host");
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
        "Generate fastest Arch Linux mirrors",
    );
    exec_eval(
        exec(
            "pacman",
            vec![
                String::from("-Syy"),
                String::from("--noconfirm"),
                String::from("--needed"),
                String::from("archlinux-keyring"),
                String::from("athena-keyring"),
                String::from("blackarch-keyring"),
                String::from("chaotic-keyring"),
            ],
        ),
        "Update keyring packages",
    );
}

fn fastest_mirrors() {
    log::info!("Getting fastest BlackArch mirrors for your location");
    exec_eval(
        exec_chroot(
            "rate-mirrors",
            vec![
                String::from("--concurrency"),
                String::from("40"),
                String::from("--disable-comments"),
                String::from("--allow-root"),
                String::from("--save"),
                String::from("/etc/pacman.d/blackarch-mirrorlist"), //In chroot we don't need to specify /mnt
                String::from("blackarch"),
            ],
        ),
        "Getting fastest mirrors from BlackArch",
    );
    log::info!("Getting fastest Chaotic AUR mirrors for your location");
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
        "Getting fastest mirrors from Chaotic AUR",
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

pub fn setting_grub_parameters() {
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_DISTRIBUTOR=.*",
            "GRUB_DISTRIBUTOR=\"Athena OS\"",
        ),
        "set distributor name",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_CMDLINE_LINUX_DEFAULT=.*",
            "GRUB_CMDLINE_LINUX_DEFAULT=\"quiet loglevel=3 audit=0 nvme_load=yes zswap.enabled=0 fbcon=nodefer nowatchdog\"",
        ),
        "set kernel parameters",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/default/grub",
            "GRUB_THEME=.*",
            "GRUB_THEME=\"/boot/grub/themes/athena/theme.txt\"",
        ),
        "enable athena grub theme",
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

pub fn install_bootloader_efi(efidir: PathBuf) {
    install::install(PackageManager::Pacman, vec![
        "grub",
        "efibootmgr",
        "os-prober",
    ]);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    log::info!("EFI bootloader installing at {}", efi_str);
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
    setting_grub_parameters();
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn install_bootloader_legacy(device: PathBuf) {
    install::install(PackageManager::Pacman, vec![
        "grub",
        "athena-grub-theme",
        "os-prober",
    ]);
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    log::info!("Legacy bootloader installing at {}", device);
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device],
        ),
        "install grub as legacy",
    );
    setting_grub_parameters();
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

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

pub fn install_homemgr() {
    install(PackageManager::Pacman, vec!["nix"]);
}

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
    enable_service("auditd");
    enable_service("bluetooth");
    enable_service("cronie");
    enable_service("NetworkManager");
    enable_service("set-cfs-tweaks");
    enable_service("ananicy");
    enable_service("irqbalance");
    //enable_service("nohang");
    enable_service("vnstat");
    //enable_service("cups");
}
