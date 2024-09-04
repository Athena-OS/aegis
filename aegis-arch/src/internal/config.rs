use crate::internal::install::install;
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
    let data = std::fs::read_to_string(&configpath);
    match &data {
        Ok(contents) => {
            files_eval(
                files::sed_file(
                    configpath.to_str().unwrap(),
                    "\"password\":.*",
                    "\"password\": \"*REDACTED*\",",
                ),
                "Redact user password hash",
            );
            files_eval(
                files::sed_file(
                    configpath.to_str().unwrap(),
                    "\"rootpass\":.*",
                    "\"rootpass\": \"*REDACTED*\",",
                ),
                "Redact root password hash",
            );
            debug!("[ \x1b[2;1;32mOK\x1b[0m ] Read config file {configpath:?}");
            // Print the contents of the config file to the install log file
            info!("Configuration set:\n{}", contents);
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
    base::install_base_packages();
    println!();
    base::install_packages(config.kernel);
    base::genfstab();
    println!();
    info!("Installing bootloader : {}", config.bootloader.r#type);
    info!("Installing bootloader to : {}", config.bootloader.location);
    if config.bootloader.r#type == "grub-efi" {
        base::install_bootloader_efi(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    } else if config.bootloader.r#type == "grub-legacy" {
        base::install_bootloader_legacy(PathBuf::from(config.bootloader.location), config.partition.encrypt_check);
    }
    println!();
    // Set locales at the beginning to prevent some warning messages about "Setting locale failed"
    info!("Adding Locales : {:?}", config.locale.locale);
    locale::set_locale(config.locale.locale.join(" "));
    info!("Using console keymap : {}", config.locale.virtkeymap);
    info!("Using x11 keymap : {}", config.locale.x11keymap);
    locale::set_keyboard(config.locale.virtkeymap.as_str(), config.locale.x11keymap.as_str());
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
    println!("---------");
    info!("Enabling zramd : {}", config.zramd);
    if config.zramd {
        base::install_zram();
    }
    println!();
    /*info!("Hardening system : {}", config.hardened);
    if config.hardened {
        secure::secure_password_config();
        secure::secure_ssh_config();
    }
    println!();*/
    info!("Installing desktop : {:?}", config.desktop);
    /*if let Some(desktop) = &config.desktop {
        desktops::install_desktop_setup(*desktop);
    }*/
    match config.desktop.to_lowercase().as_str() {
        "onyx" => desktops::install_desktop_setup(DesktopSetup::Onyx),
        "kde plasma" => desktops::install_desktop_setup(DesktopSetup::Kde), //Note that the value on this match statement must fit the name in desktops.py of aegis-gui (then they are lowercase transformed)
        "mate" => desktops::install_desktop_setup(DesktopSetup::Mate),
        "gnome" => {
            desktops::install_desktop_setup(DesktopSetup::Gnome);
            desktops::disable_xsession("gnome.desktop");
            desktops::disable_xsession("gnome-classic.desktop");
            desktops::disable_xsession("gnome-classic-xorg.desktop");
            desktops::disable_wsession("gnome.desktop");
            desktops::disable_wsession("gnome-wayland.desktop");
            desktops::disable_wsession("gnome-classic.desktop");
            desktops::disable_wsession("gnome-classic-wayland.desktop");
        },
        "cinnamon" => desktops::install_desktop_setup(DesktopSetup::Cinnamon),
        "xfce refined" => desktops::install_desktop_setup(DesktopSetup::XfceRefined),
        "xfce picom" => desktops::install_desktop_setup(DesktopSetup::XfcePicom),
        "budgie" => desktops::install_desktop_setup(DesktopSetup::Budgie),
        "enlightenment" => desktops::install_desktop_setup(DesktopSetup::Enlightenment),
        "lxqt" => desktops::install_desktop_setup(DesktopSetup::Lxqt),
        "sway" => desktops::install_desktop_setup(DesktopSetup::Sway),
        "i3" => desktops::install_desktop_setup(DesktopSetup::I3),
        "herbstluftwm" => desktops::install_desktop_setup(DesktopSetup::Herbstluftwm),
        "awesome" => desktops::install_desktop_setup(DesktopSetup::Awesome),
        "bspwm" => desktops::install_desktop_setup(DesktopSetup::Bspwm),
        "hyprland" => desktops::install_desktop_setup(DesktopSetup::Hyprland),
        "none" => desktops::install_desktop_setup(DesktopSetup::None),
        _ => info!("No desktop setup selected!"),
    }
    println!();
    info!("Installing theme : {:?}", config.theme);

    match config.theme.to_lowercase().as_str() {
        "akame" => themes::install_theme_setup(ThemeSetup::Akame),
        "cyborg" => themes::install_theme_setup(ThemeSetup::Cyborg),
        "graphite" => themes::install_theme_setup(ThemeSetup::Graphite),
        "hackthebox" => themes::install_theme_setup(ThemeSetup::HackTheBox), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "samurai" => themes::install_theme_setup(ThemeSetup::Samurai),
        "sweet" => themes::install_theme_setup(ThemeSetup::Sweet),
        "temple" => themes::install_theme_setup(ThemeSetup::Temple),
        _ => info!("No theme setup selected!"),
    }
    println!();
    info!("Installing display manager : {:?}", config.displaymanager);
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => {
            displaymanagers::install_dm_setup(DMSetup::Gdm);
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
        },
        "lightdm neon" => {
            displaymanagers::install_dm_setup(DMSetup::LightDMNeon);
            desktops::lightdm_set_session(&config.desktop);
        },
        "sddm" => displaymanagers::install_dm_setup(DMSetup::Sddm),
        _ => info!("No display manager setup selected!"),
    }

    println!();
    info!("Installing browser : {:?}", config.browser);
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
        _ => info!("No browser setup selected!"),
    }
    println!();
    // Terminal configuration //
    info!("Installing terminal : {:?}", config.terminal);
    /*if let Some(terminal) = &config.terminal {
        terminals::install_terminal_setup(*terminal);
    }*/
    let mut terminal_choice = String::new();
    match config.terminal.to_lowercase().as_str() {
        "alacritty" => {
            terminal_choice = String::from("alacritty");
            terminals::install_terminal_setup(TerminalSetup::Alacritty);
        },
        "cool retro term" => {
            terminal_choice = String::from("cool-retro-term");
            terminals::install_terminal_setup(TerminalSetup::CoolRetroTerm);
        },
        "foot" => {
            terminal_choice = String::from("foot");
            terminals::install_terminal_setup(TerminalSetup::Foot);
        },
        "gnome terminal" => {
            terminal_choice = String::from("gnome-terminal");
            terminals::install_terminal_setup(TerminalSetup::GnomeTerminal);
        },
        "kitty" => {
            terminal_choice = String::from("kitty");
            terminals::install_terminal_setup(TerminalSetup::Kitty);
        },
        "konsole" => {
            terminal_choice = String::from("konsole");
            terminals::install_terminal_setup(TerminalSetup::Konsole);
        },
        "terminator" => {
            terminal_choice = String::from("terminator");
            terminals::install_terminal_setup(TerminalSetup::Terminator);
        },
        "terminology" => {
            terminal_choice = String::from("terminology");
            terminals::install_terminal_setup(TerminalSetup::Terminology);
        },
        "urxvt" => {
            terminal_choice = String::from("urxvt");
            terminals::install_terminal_setup(TerminalSetup::Urxvt);
        },
        "xfce" => {
            terminal_choice = String::from("xfce4-terminal");
            terminals::install_terminal_setup(TerminalSetup::Xfce);
        },
        "xterm" => {
            terminal_choice = String::from("xterm");
            terminals::install_terminal_setup(TerminalSetup::Xterm);
        },
        _ => info!("No terminal setup selected!"),
    }
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
    info!("Installing flatpak : {}", config.flatpak);
    if config.flatpak {
        base::install_flatpak();
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
    // Users
    for i in 0..config.users.len() {
        info!("Creating user : {}", config.users[i].name);
        //info!("Setting user password : {}", config.users[i].password);
        info!("Enabling root for user : {}", config.users[i].hasroot);
        info!("Setting user shell : {}", config.users[i].shell);

        match config.users[i].shell.to_lowercase().as_str() {
            "bash" => shells::install_shell_setup(ShellSetup::Bash),
            "fish" => shells::install_shell_setup(ShellSetup::Fish),
            "zsh" => shells::install_shell_setup(ShellSetup::Zsh),
            _ => info!("No shell setup selected!"),
        }
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