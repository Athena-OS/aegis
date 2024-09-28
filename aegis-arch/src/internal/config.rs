use crate::internal::install::install;
use crate::internal::services::enable_service;
//use crate::internal::secure;
use crate::functions::*;
use shared::args::{self, DesktopSetup, ThemeSetup, DMSetup, ShellSetup, BrowserSetup, TerminalSetup, PackageManager, PartitionMode};
use shared::{debug, info};
use shared::exec::exec;
use shared::files;
use shared::files::sed_file;
use shared::partition;
use shared::returncode_eval::exec_eval;
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
        "goofcord-bin",
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
    //////
    let config: Config = config.unwrap();
    info!("Block device to use : {}", config.partition.device);
    info!("Partitioning mode : {:?}", config.partition.mode);
    info!("Partitioning for EFI : {}", config.partition.efi);
    info!("Swap partition : {}", config.partition.swap);
    let mut partitions: Vec<args::Partition> = Vec::new();
    for partition in config.partition.partitions {
        let to_encrypt: bool = partition.split(':').collect::<Vec<&str>>()[3].parse().map_err(|_| "Invalid boolean value").expect("Unable to get encrypt boolean value.");
        partitions.push(args::Partition::new(
            partition.split(':').collect::<Vec<&str>>()[0].to_string(),
            partition.split(':').collect::<Vec<&str>>()[1].to_string(),
            partition.split(':').collect::<Vec<&str>>()[2].to_string(),
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
    println!();

    /* BOOTLOADER PACKAGE SET */
    let boot_packages = vec![
        "grub",
        "os-prober",
        "athena-grub-theme",
    ];
    package_set.extend(boot_packages);
    if config.bootloader.r#type == "grub-efi" {
        package_set.push("efibootmgr");
    }
    /**************************/
    println!();
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
        "xfce refined" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::XfceRefined)),
        "xfce picom" => package_set.extend(desktops::install_desktop_setup(DesktopSetup::XfcePicom)),
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
    info!("Selecting display manager : {:?}", config.displaymanager);
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::Gdm)),
        "lightdm neon" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::LightDMNeon)),
        "sddm" => package_set.extend(displaymanagers::install_dm_setup(DMSetup::Sddm)),
        _ => info!("No display manager setup selected!"),
    }
    /**************************/
    println!();
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
    println!();
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
    println!();
    /*         THEME         */
    info!("Selecting theme : {:?}", config.theme);
    match config.theme.to_lowercase().as_str() {
        "akame" => package_set.extend(themes::install_theme_setup(ThemeSetup::Akame)),
        "cyborg" => package_set.extend(themes::install_theme_setup(ThemeSetup::Cyborg)),
        "graphite" => package_set.extend(themes::install_theme_setup(ThemeSetup::Graphite)),
        "hackthebox" => package_set.extend(themes::install_theme_setup(ThemeSetup::HackTheBox)), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "samurai" => package_set.extend(themes::install_theme_setup(ThemeSetup::Samurai)),
        "sweet" => package_set.extend(themes::install_theme_setup(ThemeSetup::Sweet)),
        "temple" => package_set.extend(themes::install_theme_setup(ThemeSetup::Temple)),
        _ => info!("No theme setup selected!"),
    }
    /**************************/
    println!();
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
    println!();
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
    println!();
    /********** INSTALLATION **********/

    base::install_packages(config.kernel, package_set);

    /**************************/
    println!();
    /********** CONFIGURATION **********/

    base::genfstab();
    println!();
    info!("Configuring bootloader : {}", config.bootloader.r#type);
    info!("Configuring bootloader to : {}", config.bootloader.location);
    if config.bootloader.r#type == "grub-efi" {
        base::configure_bootloader_efi(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    } else if config.bootloader.r#type == "grub-legacy" {
        base::configure_bootloader_legacy(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    }
    println!();
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
    info!("Processing all presets.");
    base::preset_process();
    println!();
    info!("Hostname : {}", config.networking.hostname);
    network::set_hostname(config.networking.hostname.as_str());
    network::create_hosts();
    info!("Enabling ipv6 : {}", config.networking.ipv6);
    if config.networking.ipv6 {
        network::enable_ipv6();
    }
    println!();
    if config.zramd {
        info!("Enabling zramd : {}", config.zramd);
        base::configure_zram();
    }
    println!();
    /*info!("Hardening system : {}", config.hardened);
    if config.hardened {
        secure::secure_password_config();
        secure::secure_ssh_config();
    }
    println!();*/
    info!("Configuring desktop : {:?}", config.desktop);
    match config.desktop.to_lowercase().as_str() {
        "gnome" => {
            desktops::disable_xsession("gnome.desktop");
            desktops::disable_xsession("gnome-classic.desktop");
            desktops::disable_xsession("gnome-classic-xorg.desktop");
            desktops::disable_wsession("gnome.desktop");
            desktops::disable_wsession("gnome-wayland.desktop");
            desktops::disable_wsession("gnome-classic.desktop");
            desktops::disable_wsession("gnome-classic-wayland.desktop");
        },
        _ => info!("No desktop configuration needed."),
    }
    println!();
    info!("Configuring display manager : {:?}", config.displaymanager);
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => {
            if ! config.desktop.contains("gnome") {
                files::rename_file("/mnt/usr/lib/udev/rules.d/61-gdm.rules", "/mnt/usr/lib/udev/rules.d/61-gdm.rules.bak");
                desktops::disable_xsession("gnome.desktop");
                desktops::disable_xsession("gnome-xorg.desktop");
                desktops::disable_wsession("gnome.desktop");
                desktops::disable_wsession("gnome-wayland.desktop");
                // Note that gnome-classic sessions belong to gnome-shell-extensions pkg that is not installed by GDM
            }
            else {
                files_eval(
                    files::sed_file(
                        "/mnt/etc/gdm/custom.conf",
                        ".*WaylandEnable=.*",
                        "WaylandEnable=false",
                    ),
                    "Disable Wayland in GNOME",
                );
            }
            enable_service("gdm");
        },
        "lightdm neon" => {
            desktops::lightdm_set_session(&config.desktop);
            enable_service("lightdm");
        },
        "sddm" => enable_service("sddm"),
        _ => info!("No display manager configuration needed."),
    }

    println!();
    info!("Configuring browser : {:?}", config.browser);
    /*if let Some(browser) = &config.browser {
        browsers::install_browser_setup(*browser);
    }*/
    match config.browser.to_lowercase().as_str() {
        "firefox" => {
            browsers::install_browser_setup(BrowserSetup::Firefox);
            if config.desktop.contains("gnome") {
                files_eval(
                    files::sed_file(
                        "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                        "\\{\\\\\"name\\\\\":\\\\\"Brave\\\\\",\\\\\"icon\\\\\":\\\\\"/usr/share/icons/hicolor/scalable/apps/brave.svg\\\\\",\\\\\"type\\\\\":\\\\\"Command\\\\\",\\\\\"data\\\\\":\\{\\\\\"command\\\\\":\\\\\"brave\\\\\"\\},\\\\\"angle\\\\\":-1\\}",
                        "{\\\"name\\\":\\\"Firefox ESR\\\",\\\"icon\\\":\\\"/usr/share/icons/hicolor/scalable/apps/firefox-logo.svg\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{\\\"command\\\":\\\"firefox-esr\\\"},\\\"angle\\\":-1}",
                    ),
                    "Apply Browser info on dconf shell",
                );
            }
        },
        "brave" => {
            browsers::install_browser_setup(BrowserSetup::Brave);
            if config.desktop.contains("gnome") {
                files_eval(
                    files::sed_file(
                        "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                        "\\{\\\\\"name\\\\\":\\\\\"Firefox ESR\\\\\",\\\\\"icon\\\\\":\\\\\"/usr/share/icons/hicolor/scalable/apps/firefox-logo.svg\\\\\",\\\\\"type\\\\\":\\\\\"Command\\\\\",\\\\\"data\\\\\":\\{\\\\\"command\\\\\":\\\\\"firefox-esr\\\\\"\\},\\\\\"angle\\\\\":-1\\}",
                        "{\\\"name\\\":\\\"Brave\\\",\\\"icon\\\":\\\"/usr/share/icons/hicolor/scalable/apps/brave.svg\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{\\\"command\\\":\\\"brave\\\"},\\\"angle\\\":-1}",
                    ),
                    "Apply Browser info on dconf shell",
                );
            }
        }
        _ => info!("No browser configuration needed."),
    }
    println!();
    info!("Configuring theme : {:?}", config.theme);
    match config.theme.to_lowercase().as_str() {
        "akame" => themes::configure_akame(),
        "cyborg" => themes::configure_cyborg(),
        "graphite" => themes::configure_graphite(),
        "hackthebox" => themes::configure_hackthebox(),
        "samurai" => themes::configure_samurai(),
        "sweet" => themes::configure_sweet(),
        "temple" => themes::configure_temple(),
        _ => info!("No theme configuration needed."),
    }
    println!();
    //////////
    exec_eval(
        exec( // Using exec instead of exec_chroot because in exec_chroot, these sed arguments need some chars to be escaped
            "sed",
            vec![
                String::from("-i"),
                String::from("-e"),
                format!("s/^TERMINAL_EXEC=.*/TERMINAL_EXEC=\"{}\"/g", &(terminal_choice.clone()+" "+if terminal_choice == "gnome-terminal" { "--" } else { "-e" })),
                String::from("/mnt/usr/bin/shell-rocket"),
            ],
        ),
        "Set terminal on shell rocket",
    );
    files_eval(
        sed_file(
            "/mnt/usr/share/applications/shell.desktop",
            "alacritty",
            &terminal_choice,
        ),
        "Set terminal call on shell.desktop file",
    );
    if config.desktop.contains("gnome") {
        files_eval(
            sed_file(
                "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                "alacritty",
                &terminal_choice,
            ),
            "Set terminal call on dconf file",
        );
    }
    // Misc Settings
    println!();
    /*info!("Installing snapper : {}", config.snapper);
    if config.snapper {
        base::setup_snapper();
    }
    println!();*/
    if config.flatpak {
        info!("Configuring flatpak : {}", config.flatpak);
        base::configure_flatpak();
    }
    info!("Extra packages : {:?}", config.extra_packages);
    let mut extra_packages: Vec<&str> = Vec::new();
    for i in 0..config.extra_packages.len() {
        extra_packages.push(config.extra_packages[i].as_str());
    }
    install(PackageManager::Pacman, extra_packages);
    println!();
    info!("Enabling system services...");
    base::enable_system_services();
    println!("---------");
    // SHELL
    // The shell of the first created user will be applied on shell.desktop and on SHELL variable
    match config.users[0].shell.to_lowercase().as_str() {
        "fish" => {
            files_eval(
                files::sed_file(
                    "/mnt/usr/share/applications/shell.desktop",
                    "Bash",
                    "Fish",
                ),
                "Apply FISH shell on .desktop shell file",
            );
            files_eval(
                files::sed_file(
                    "/mnt/etc/skel/.bashrc",
                    "export SHELL=.*",
                    r"export SHELL=$(which fish)",
                ),
                "Apply FISH shell",
            );
        },
        "zsh" => {
            files_eval(
                files::sed_file(
                    "/mnt/usr/share/applications/shell.desktop",
                    "Bash",
                    "Zsh",
                ),
                "Apply ZSH shell on .desktop shell file",
            );
            files_eval(
                files::sed_file(
                    "/mnt/etc/skel/.bashrc",
                    "export SHELL=.*",
                    r"export SHELL=$(which zsh)",
                ),
                "Apply ZSH shell",
            );
        },
        _ => info!("No shell configuration needed."),
    }
    // Users
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
        println!("---------");
    }
    println!();
    //info!("Setting root password : {}", config.rootpass);
    users::root_pass(config.rootpass.as_str());
    println!();
    info!("Installation log file copied to /var/log/aegis.log");
    files_eval(files::create_directory("/mnt/var/log"), "create /mnt/var/log");
    files::copy_file("/tmp/aegis.log", "/mnt/var/log/aegis.log");
    if config.bootloader.r#type == "grub-efi" {
        partition::umount("/mnt/boot");
    }
    partition::umount("/mnt/home");
    partition::umount("/mnt");
    println!("Installation finished! You may reboot now!");
    0
}