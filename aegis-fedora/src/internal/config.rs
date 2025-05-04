//use crate::internal::secure;
use crate::functions::*;
use crate::internal::secure;
use shared::args::{self, DesktopSetup, ThemeSetup, DMSetup, ShellSetup, BrowserSetup, TerminalSetup, PartitionMode};
use shared::{debug, info};
use shared::files;
use shared::partition;
use shared::returncode_eval::files_eval;
use shared::serde::{self, Deserialize, Serialize};
use shared::serde_json;
use shared::strings::crash;
use std::path::{PathBuf};


#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Config {
    partition: Partition,
    bootloader: Bootloader,
    locale: Locale,
    networking: Networking,
    users: Vec<Users>,
    rootpass: String,
    desktop: String,
    theme: String,
    displaymanager: String,
    browser: String,
    terminal: String,
    //snapper: bool,
    flatpak: bool,
    zramd: bool,
    //hardened: bool,
    extra_packages: Vec<String>,
    kernel: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Partition {
    device: String,
    mode: PartitionMode,
    encrypt_check: bool,
    efi: bool,
    swap: bool,
    swap_size: String,
    partitions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Bootloader {
    r#type: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Locale {
    locale: Vec<String>,
    virtkeymap: String,
    x11keymap: String,
    timezone: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Networking {
    hostname: String,
    ipv6: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")] // must be below the derive attribute
struct Users {
    name: String,
    password: String,
    hasroot: bool,
    shell: String,
}

pub fn read_config(configpath: PathBuf) -> i32 {
    let mut package_set: Vec<&str> = vec![
        "NetworkManager",
        "network-manager-applet",
        "man-db",
        "man-pages",
        "nano",
        "sudo",
        "curl",
        "dnf5",
        "e2fsprogs", // needed to run chattr on post install scripts
        // Extra Base Fedora
        "accountsservice",
        "alacritty",
        "alsa-sof-firmware",
        "alsa-utils",
        "audit",
        "bind",
        "bluez",
        "cracklib-dicts",
        "dhcpcd",
        "dialog",
        "dosfstools",
        "iproute",
        "irqbalance",
        "lvm2",
        "memtest86+",
        "mesa-dri-drivers",
        "mesa-vulkan-drivers",
        "most",
        "mtools",
        "nbd",
        "net-tools",
        "nfs-utils",
        "nss-mdns",
        "ntfsprogs",
        "ntpsec",
        "pavucontrol",
        "pv",
        "rsync",
        "selinux-policy-targeted",
        "squashfs-tools",
        "syslinux",
        "terminus-fonts-console",
        "testdisk",
        "usbutils",
        "wpa_supplicant",
        "xfsprogs",
        // Fonts
        "google-noto-fonts-common",
        "google-noto-color-emoji-fonts",
        "google-noto-sans-cjk-fonts",
        // Common packages for all desktops
        "pipewire",
        "pipewire-pulseaudio",
        "pipewire-alsa",
        "pipewire-jack-audio-connection-kit",
        "wireplumber",
        "ntfs-3g",
        //"eza", //Fedora 41
        "pocl", // Hashcat dependency
        "asciinema",
        "btop",
        "bat",
        "bc",
        "bless",
        "cmatrix",
        "cowsay",
        "cronie",
        "eog",
        "espeak",
        "fastfetch",
        "figlet",
        "file-roller",
        "fortune-mod",
        "git",
        "gparted",
        //"grub-customizer", //Fedora 41
        "gtk-murrine-engine",
        "gvfs-gphoto2",
        "gvfs-mtp",
        "hexedit",
        //"hw-probe, //HW probing
        "ImageMagick",
        "jq",
        "keepassxc",
        "lolcat",
        "lsd",
        "nano-syntax-highlighting",
        "nautilus",
        "ncdu",
        "NetworkManager-openvpn",
        "neovim-nvchad",
        "netcat",
        "nyancat",
        "onionshare",
        "openvpn",
        "orca",
        "p7zip",
        "polkit",
        "sl",
        //"smartmontools", //hw-probe deps
        "tidy",
        "torbrowser-launcher",
        "tree",
        "ufw",
        "unzip",
        "vnstat",
        "wget2-wget",
        "which",
        "xclip",
        "xmlstarlet",
        "zoxide",
        // Athena
        //"athena-cyber-hub",
        //"athena-neofetch-config",
        //"athena-powershell-config",
        "athena-config",
        "athena-tmux-config",
        "athena-tweak-tool",
        "athena-vscodium-themes",
        "athena-welcome",
        "htb-toolkit",
        "kando",
        "nist-feed",
    ];

    let data = std::fs::read_to_string(&configpath);
    match &data {
        Ok(_) => {
            files_eval(
                files::sed_file(
                    configpath.to_str().unwrap(),
                    "\"password\":.*",
                    "\"password\": \"*REDACTED*\",",
                ),
                "Redact user information",
            );
            files_eval(
                files::sed_file(
                    configpath.to_str().unwrap(),
                    "\"rootpass\":.*",
                    "\"rootpass\": \"*REDACTED*\",",
                ),
                "Redact root information",
            );
            // Re-read the redacted file content
            let redacted_data = std::fs::read_to_string(&configpath).expect("Failed to read config file after redaction");
            info!("Configuration set:\n{}", redacted_data);

            debug!("[ \x1b[2;1;32mOK\x1b[0m ] Read and redacted config file {configpath:?}");
        }
        Err(e) => {
            crash(
                format!("Read config file {configpath:?}  ERROR: {}", e),
                e.raw_os_error().unwrap(),
            );
        }
    }
    let config: std::result::Result<Config, serde_json::Error> =
        serde_json::from_str(&data.unwrap());
    match &config {
        Ok(_) => {
            debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parse config file {configpath:?}",);
        }
        Err(e) => {
            crash(format!("Parse config file {configpath:?}  ERROR: {}", e), 1);
        }
    }
    /*****TEMPORARY DISABLE SELINUX*****/
    if secure::selinux_enabled() {
        secure::set_selinux_mode("0");
    }
    /*    PARTITIONING    */
    let config: Config = config.unwrap();
    info!("Block device to use : {}", config.partition.device);
    info!("Partitioning mode : {:?}", config.partition.mode);
    info!("Partitioning for EFI : {}", config.partition.efi);
    info!("Swap partition : {}", config.partition.swap);
    let mut partitions: Vec<args::Partition> = Vec::new();
    for partition in config.partition.partitions {
        let to_encrypt: bool = partition.split(':').collect::<Vec<&str>>()[4].parse().map_err(|_| "Invalid boolean value").expect("Unable to get encrypt boolean value.");
        partitions.push(args::Partition::new(
            partition.split(':').collect::<Vec<&str>>()[0].to_string(),
            partition.split(':').collect::<Vec<&str>>()[1].to_string(),
            partition.split(':').collect::<Vec<&str>>()[2].to_string(),
            partition.split(':').collect::<Vec<&str>>()[3].to_string(),
            to_encrypt,
        ));
    }
    let device = PathBuf::from("/dev/").join(config.partition.device.as_str());
    partition::partition(
        device,
        config.partition.mode,
        config.partition.encrypt_check,
        config.partition.efi,
        config.partition.swap,
        config.partition.swap_size,
        &mut partitions,
    );

    /* BOOTLOADER PACKAGE SET */
    let boot_packages = vec![
        "grub2",
        "os-prober",
        //"athena-grub-theme",
    ];
    package_set.extend(boot_packages);
    if config.bootloader.r#type == "grub-efi" {
        package_set.push("efibootmgr");
        package_set.push("grub2-efi");
        package_set.push("grub2-efi-x64-modules"); // Not sure if it works also for ARM CPU
        package_set.push("shim-\*");
    }
    /**************************/
    /*        DESKTOP         */
    info!("Selected desktop : {:?}", config.desktop);
    /*if let Some(desktop) = &config.desktop {
        desktops::install_desktop_setup(*desktop);
    }*/
    match config.desktop.to_lowercase().as_str() {
        "onyx" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Onyx)),
        "kde plasma" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Kde)), //Note that the value on this match statement must fit the name in desktops.py of aegis-gui (then they are lowercase transformed)
        "mate" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Mate)),
        "gnome" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Gnome)),
        "cinnamon" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Cinnamon)),
        "xfce picom" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::XfcePicom)),
        "xfce refined" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::XfceRefined)),
        "budgie" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Budgie)),
        "enlightenment" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Enlightenment)),
        "lxqt" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Lxqt)),
        "sway" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Sway)),
        "i3" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::I3)),
        "herbstluftwm" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Herbstluftwm)),
        "awesome" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Awesome)),
        "bspwm" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Bspwm)),
        "hyprland" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::Hyprland)),
        "none" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::None)),
        _ => info!("No desktop setup selected!"),
    }
    /**************************/

    /*     DISPLAY MANAGER    */
    info!("Selected display manager : {:?}", config.displaymanager);
    package_set.extend(displaymanagers::install_dm_setup(DMSetup::Sddm));
    //match config.displaymanager.to_lowercase().as_str() {
    //    "gdm" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::Gdm)),
    //    "lightdm neon" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::LightDMNeon)),
    //    "sddm" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::Sddm)),
    //    _ => info!("No display manager setup selected!"),
    //}
    /**************************/
    /* BROWSER PACKAGE SET */
    info!("Selected browser : {:?}", config.browser);
    match config.browser.to_lowercase().as_str() {
        "firefox" => {
            package_set.extend(browsers::install_browser_setup(BrowserSetup::Firefox));
        },
        "brave" => {
            package_set.extend(browsers::install_browser_setup(BrowserSetup::Brave));
        },
        _ => info!("No browser setup selected!"),
    }
    /**************************/
    /*        TERMINAL       */
    info!("Selected terminal : {:?}", config.terminal);
    let mut terminal_choice = String::new();
    match config.terminal.to_lowercase().as_str() {
        "alacritty" => {
            terminal_choice = String::from("alacritty");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Alacritty));
        },
        "cool retro term" => {
            terminal_choice = String::from("cool-retro-term");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::CoolRetroTerm));
        },
        "foot" => {
            terminal_choice = String::from("foot");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Foot));
        },
        "gnome terminal" => {
            terminal_choice = String::from("gnome-terminal");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::GnomeTerminal));
        },
        "kitty" => {
            terminal_choice = String::from("kitty");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Kitty));
        },
        "konsole" => {
            terminal_choice = String::from("konsole");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Konsole));
        },
        "terminator" => {
            terminal_choice = String::from("terminator");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Terminator));
        },
        "terminology" => {
            terminal_choice = String::from("terminology");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Terminology));
        },
        "urxvt" => {
            terminal_choice = String::from("urxvt");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Urxvt));
        },
        "xfce" => {
            terminal_choice = String::from("xfce4-terminal");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Xfce));
        },
        "xterm" => {
            terminal_choice = String::from("xterm");
            package_set.extend(terminals::install_terminal_setup(TerminalSetup::Xterm));
        },
        _ => info!("No terminal setup selected!"),
    }
    /**************************/
    /*         THEME         */
    info!("Selected theme : {:?}", config.theme);
    match config.theme.to_lowercase().as_str() {
        "cyborg" => package_set.extend(themes::install_theme_setup(ThemeSetup::Cyborg)),
        "graphite" => package_set.extend(themes::install_theme_setup(ThemeSetup::Graphite)),
        "hackthebox" => package_set.extend(themes::install_theme_setup(ThemeSetup::HackTheBox)), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "redmoon" => package_set.extend(themes::install_theme_setup(ThemeSetup::RedMoon)),
        "samurai" => package_set.extend(themes::install_theme_setup(ThemeSetup::Samurai)),
        "sweet" => package_set.extend(themes::install_theme_setup(ThemeSetup::Sweet)),
        "temple" => package_set.extend(themes::install_theme_setup(ThemeSetup::Temple)),
        _ => info!("No theme setup selected!"),
    }
    /**************************/
    /*          MISC         */

    if config.zramd {
        info!("Selecting zramd : {}", config.zramd);
        package_set.push("zram-generator");
    }
    if config.flatpak {
        info!("Selecting flatpak : {}", config.flatpak);
        package_set.push("flatpak");
    }
    /**************************/
    /*         USERS         */
    for i in 0..config.users.len() {
        match config.users[i].shell.to_lowercase().as_str() {
            "bash" => package_set.extend(shells::install_shell_setup(ShellSetup::Bash)),
            "fish" => package_set.extend(shells::install_shell_setup(ShellSetup::Fish)),
            "zsh" => package_set.extend(shells::install_shell_setup(ShellSetup::Zsh)),
            _ => info!("No shell setup selected!"),
        }
    }
    /**************************/
    /********** INSTALLATION **********/

    base::install_packages(package_set);

    /**************************/
    /********** CONFIGURATION **********/

    base::genfstab();

    /*    BOOTLOADER CONFIG     */
    info!("Configuring bootloader : {}", config.bootloader.r#type);
    info!("Configuring bootloader to : {}", config.bootloader.location);
    if config.bootloader.r#type == "grub-efi" {
        base::configure_bootloader_efi(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    } else if config.bootloader.r#type == "grub-legacy" {
        base::configure_bootloader_legacy(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    }
    /**************************/
    /*         LOCALES        */
    // Set locales at the beginning to prevent some warning messages about "Setting locale failed"
    info!("Adding Locales : {:?}", config.locale.locale);
    locale::set_locale(config.locale.locale.join(" "));
    info!("Using console keymap : {}", config.locale.virtkeymap);
    info!("Using x11 keymap : {}", config.locale.x11keymap);
    locale::set_keyboard(config.locale.virtkeymap.as_str(), config.locale.x11keymap.as_str())
        .unwrap_or_else(|e| {
            eprintln!("Error setting keyboard configuration: {}", e);
        });
    info!("Setting timezone : {}", config.locale.timezone);
    locale::set_timezone(config.locale.timezone.as_str());
    /**************************/
    info!("Hostname : {}", config.networking.hostname);
    network::set_hostname(config.networking.hostname.as_str());
    network::create_hosts();
    files::copy_multiple_files("/etc/NetworkManager/system-connections/*", "/mnt/etc/NetworkManager/system-connections/");
    /**************************/
    /*     DESKTOP CONFIG     */
    info!("Configuring desktop : {:?}", config.desktop);
    match config.desktop.to_lowercase().as_str() {
        "gnome" => desktops::configure_gnome(),
        "hyprland" => desktops::configure_hyprland(),
        //"xfce picom" => desktops::configure_xfce(),
        //"xfce refined" => desktops::configure_xfce(),
        _ => info!("No desktop configuration needed."),
    }
    /**************************/
    /* DISPLAY MANAGER CONFIG */
    info!("Configuring display manager : {:?}", config.displaymanager);
    displaymanagers::configure_sddm();
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => displaymanagers::configure_gdm(&config.desktop),
        "lightdm neon" => displaymanagers::configure_lightdm_neon(&config.desktop),
        "sddm" => displaymanagers::configure_sddm(),
        "astronaut" => displaymanagers::configure_sddm_astronaut(),
        "black hole" => displaymanagers::configure_sddm_blackhole(),
        "cyberpunk" => displaymanagers::configure_sddm_cyberpunk(),
        "cyborg" => displaymanagers::configure_sddm_cyborg(),
        "jake the dog" => displaymanagers::configure_sddm_jake(),
        "kath" => displaymanagers::configure_sddm_kath(),
        "pixel sakura" => displaymanagers::configure_sddm_pixelsakura(),
        "post-apocalypse" => displaymanagers::configure_sddm_postapocalypse(),
        "purple leaves" => displaymanagers::configure_sddm_purpleleaves(),
        _ => info!("No display manager configuration needed."),
    }
    /**************************/
    /*    TERMINAL CONFIG    */
    info!("Configuring terminal : {}", config.terminal);
    terminals::configure_terminal(terminal_choice);
    /**************************/
    /*      THEME CONFIG     */
    info!("Configuring theme : {:?}", config.theme);
    match config.theme.to_lowercase().as_str() {
        "cyborg" => themes::configure_cyborg(),
        "graphite" => themes::configure_graphite(),
        "hackthebox" => themes::configure_hackthebox(),
        "redmoon" => themes::configure_redmoon(),
        "samurai" => themes::configure_samurai(),
        "sweet" => themes::configure_sweet(),
        "temple" => themes::configure_temple(),
        _ => info!("No theme configuration needed."),
    }
    /**************************/

    /*info!("Installing snapper : {}", config.snapper);
    if config.snapper {
        base::setup_snapper();
    }*/
    
    /*    EXTRA PACKAGES    */
    /*
    info!("Extra packages : {:?}", config.extra_packages);
    let mut extra_packages: Vec<&str> = Vec::new();
    for i in 0..config.extra_packages.len() {
        extra_packages.push(config.extra_packages[i].as_str());
    }
    install(PackageManager::Dnf, extra_packages);
    */
    /**************************/
    /*     SHELL CONFIG     */
    // The shell of the first created user will be applied on shell.desktop and on SHELL variable
    match config.users[0].shell.to_lowercase().as_str() {
        "fish" => shells::configure_fish(),
        "zsh" => shells::configure_zsh(),
        _ => info!("No shell configuration needed."),
    }
    /**************************/
    /*          MISC         */
    info!("Enabling ipv6 : {}", config.networking.ipv6);
    if config.networking.ipv6 {
        network::enable_ipv6();
    }
    if config.zramd {
        info!("Enabling zramd : {}", config.zramd);
        base::configure_zram();
    }
    if config.flatpak {
        info!("Configuring flatpak : {}", config.flatpak);
        base::configure_flatpak();
    }
    /*info!("Hardening system : {}", config.hardened);
    if config.hardened {
        secure::secure_password_config();
        secure::secure_ssh_config();
    }*/
    /**************************/
    /*      USER CONFIG      */
    for i in 0..config.users.len() {
        info!("Creating user : {}", config.users[i].name);
        //info!("Setting user password : {}", config.users[i].password);
        info!("Enabling root for user : {}", config.users[i].hasroot);
        info!("Setting user shell : {}", config.users[i].shell);

        users::new_user(
            config.users[i].name.as_str(),
            config.users[i].hasroot,
            config.users[i].password.as_str(),
            false,
            "bash", //config.users[i].shell.as_str(), // Use bash because it must be the shell associated to the user in order to source the initial .sh files at login time
        );
    }
    //info!("Setting root password : {}", config.rootpass);
    users::root_pass(config.rootpass.as_str());
    /**************************/
    /*    ENABLE SERVICES    */
    info!("Enabling system services...");
    base::enable_system_services();
    /**************************/
    /*   SET SELINUX CONTEXT   */
    info!("Applying security labels on files...");
    secure::set_security_context();
    /**************************/
    info!("Installation log file copied to /var/log/aegis.log");
    files_eval(files::create_directory("/mnt/var/log"), "create /mnt/var/log");
    files::copy_file("/tmp/aegis.log", "/mnt/var/log/aegis.log");
    if config.bootloader.r#type == "grub-efi" {
        partition::umount("/mnt/boot");
    }
    partition::umount("/mnt/home");
    partition::umount("/mnt");
    if secure::selinux_enabled() {
        secure::set_selinux_mode("1");
    }
    info!("Installation finished! You may reboot now!");
    0
}
