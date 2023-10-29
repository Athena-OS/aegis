use crate::args;
use crate::args::{DesktopSetup, ThemeSetup, DMSetup, ShellSetup, BrowserSetup, TerminalSetup, PartitionMode, PackageManager};
use crate::functions::*;
use crate::internal::*;
use crate::internal::files::sed_file;
use crate::internal::secure;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf};


#[derive(Serialize, Deserialize)]
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
    timeshift: bool,
    snapper: bool,
    flatpak: bool,
    zramd: bool,
    hardened: bool,
    extra_packages: Vec<String>,
    kernel: String,
}

#[derive(Serialize, Deserialize)]
struct Partition {
    device: String,
    mode: PartitionMode,
    efi: bool,
    partitions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Bootloader {
    r#type: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
struct Locale {
    locale: Vec<String>,
    keymap: String,
    timezone: String,
}

#[derive(Serialize, Deserialize)]
struct Networking {
    hostname: String,
    ipv6: bool,
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
    password: String,
    hasroot: bool,
    shell: String,
}

pub fn read_config(configpath: PathBuf) {
    let data = std::fs::read_to_string(&configpath);
    match &data {
        Ok(_) => {
            log::debug!("[ \x1b[2;1;32mOK\x1b[0m ] Read config file {configpath:?}");
        }
        Err(e) => {
            crash(
                format!("Read config file {configpath:?}  ERROR: {}", e),
                e.raw_os_error().unwrap(),
            );
        }
    }
    /*let config: std::result::Result<Config, toml::de::Error> =
        toml::from_str(&data.unwrap());
    match &config {
        Ok(_) => {
            log::debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parse config file {configpath:?}",);
        }
        Err(e) => {
            crash(format!("Parse config file {configpath:?}  ERROR: {}", e), 1);
        }
    }*/
    /////// USED ONLY FOR TESTING PURPOSES
    let config: std::result::Result<Config, serde_json::Error> =
        serde_json::from_str(&data.unwrap());
    match &config {
        Ok(_) => {
            log::debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parse config file {configpath:?}",);
        }
        Err(e) => {
            crash(format!("Parse config file {configpath:?}  ERROR: {}", e), 1);
        }
    }
    //////
    let config: Config = config.unwrap();
    log::info!("Block device to use : {}", config.partition.device);
    log::info!("Partitioning mode : {:?}", config.partition.mode);
    log::info!("Partitioning for EFI : {}", config.partition.efi);
    let mut partitions: Vec<args::Partition> = Vec::new();
    for partition in config.partition.partitions {
        partitions.push(args::Partition::new(
            partition.split(':').collect::<Vec<&str>>()[0].to_string(),
            partition.split(':').collect::<Vec<&str>>()[1].to_string(),
            partition.split(':').collect::<Vec<&str>>()[2].to_string(),
        ));
    }
    let device = PathBuf::from("/dev/").join(config.partition.device.as_str());
    partition::partition(
        device,
        config.partition.mode,
        config.partition.efi,
        &mut partitions,
    );
    println!();
    base::install_base_packages();
    println!();
    // Set locales at the beginning to prevent some warning messages about "Setting locale failed"
    log::info!("Adding Locales : {:?}", config.locale.locale);
    locale::set_locale(config.locale.locale.join(" "));
    log::info!("Using keymap : {}", config.locale.keymap);
    locale::set_keyboard(config.locale.keymap.as_str());
    log::info!("Setting timezone : {}", config.locale.timezone);
    locale::set_timezone(config.locale.timezone.as_str());
    println!();
    base::install_packages(config.kernel);
    base::genfstab();
    println!();
    log::info!("Installing bootloader : {}", config.bootloader.r#type);
    log::info!("Installing bootloader to : {}", config.bootloader.location);
    if config.bootloader.r#type == "grub-efi" {
        base::install_bootloader_efi(PathBuf::from(config.bootloader.location));
    } else if config.bootloader.r#type == "grub-legacy" {
        base::install_bootloader_legacy(PathBuf::from(config.bootloader.location));
    }
    println!();
    log::info!("Hostname : {}", config.networking.hostname);
    log::info!("Enabling ipv6 : {}", config.networking.ipv6);
    network::set_hostname(config.networking.hostname.as_str());
    network::create_hosts();
    if config.networking.ipv6 {
        network::enable_ipv6();
    }
    println!();
    println!("---------");
    log::info!("Enabling zramd : {}", config.zramd);
    if config.zramd {
        base::install_zram();
    }
    println!();
    log::info!("Hardening system : {}", config.hardened);
    if config.hardened {
        secure::secure_password_config();
        secure::secure_ssh_config();
    }
    println!();
    log::info!("Installing desktop : {:?}", config.desktop);
    /*if let Some(desktop) = &config.desktop {
        desktops::install_desktop_setup(*desktop);
    }*/
    match config.desktop.to_lowercase().as_str() {
        "onyx" => desktops::install_desktop_setup(DesktopSetup::Onyx),
        "kde plasma" => desktops::install_desktop_setup(DesktopSetup::Kde), //Note that the value on this match statement must fit the name in desktops.py of aegis-gui (then they are lowercase transformed)
        "mate" => desktops::install_desktop_setup(DesktopSetup::Mate),
        "gnome" => {
            desktops::install_desktop_setup(DesktopSetup::Gnome);
            disable_xsession("gnome.desktop");
            disable_xsession("gnome-classic.desktop");
            disable_xsession("gnome-classic-xorg.desktop");
            disable_wsession("gnome.desktop");
            disable_wsession("gnome-classic.desktop");
            disable_wsession("gnome-classic-wayland.desktop");
        },
        "cinnamon" => desktops::install_desktop_setup(DesktopSetup::Cinnamon),
        "xfce well" => desktops::install_desktop_setup(DesktopSetup::XfceWell),
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
        "none/diy" => desktops::install_desktop_setup(DesktopSetup::None),
        _ => log::info!("No desktop setup selected!"),
    }
    println!();
    log::info!("Installing theme : {:?}", config.theme);
    /*if let Some(theme) = &config.theme {
        themes::install_theme_setup(*theme);
    }*/
    match config.theme.to_lowercase().as_str() {
        "akame" => themes::install_theme_setup(ThemeSetup::Akame),
        "cyborg" => themes::install_theme_setup(ThemeSetup::Cyborg),
        "everblush" => themes::install_theme_setup(ThemeSetup::Everblush),
        "graphite" => themes::install_theme_setup(ThemeSetup::Graphite),
        "hackthebox" => themes::install_theme_setup(ThemeSetup::HackTheBox), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "samurai" => themes::install_theme_setup(ThemeSetup::Samurai),
        "sweet" => themes::install_theme_setup(ThemeSetup::Sweet),
        "xxe" => themes::install_theme_setup(ThemeSetup::Xxe),
        _ => log::info!("No theme setup selected!"),
    }
    println!();
    log::info!("Installing display manager : {:?}", config.displaymanager);
    /*if let Some(displaymanager) = &config.displaymanager {
        displaymanagers::install_displaymanager_setup(*displaymanager);
    }*/
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => {
            displaymanagers::install_dm_setup(DMSetup::Gdm);
            if ! config.desktop.contains("gnome") {
                files::rename_file("/mnt/usr/lib/udev/rules.d/61-gdm.rules", "/mnt/usr/lib/udev/rules.d/61-gdm.rules.bak");
                disable_xsession("gnome.desktop");
                disable_xsession("gnome-xorg.desktop");
                disable_wsession("gnome.desktop");
                disable_wsession("gnome-wayland.desktop");
                // Note that gnome-classic sessions belong to gnome-shell-extensions pkg that is not installed by GDM
            }
        },
        "lightdm neon" => {
            displaymanagers::install_dm_setup(DMSetup::LightDMNeon);
            lightdm_set_session(&config.desktop);
        },
        "lightdm everblush" => {
            displaymanagers::install_dm_setup(DMSetup::LightDMEverblush);
            lightdm_set_session(&config.desktop);
        },
        "sddm" => displaymanagers::install_dm_setup(DMSetup::Sddm),
        _ => log::info!("No display manager setup selected!"),
    }

    println!();
    log::info!("Installing browser : {:?}", config.browser);
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
        "mullvad" => {
            browsers::install_browser_setup(BrowserSetup::Mullvad);
            if config.desktop.contains("gnome") {
                files_eval(
                    files::sed_file(
                        "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                        "\\{\\\\\"name\\\\\":\\\\\"Firefox ESR\\\\\",\\\\\"icon\\\\\":\\\\\"/usr/share/icons/hicolor/scalable/apps/firefox-logo.svg\\\\\",\\\\\"type\\\\\":\\\\\"Command\\\\\",\\\\\"data\\\\\":\\{\\\\\"command\\\\\":\\\\\"firefox-esr\\\\\"\\},\\\\\"angle\\\\\":-1\\}",
                        "{\\\"name\\\":\\\"Mullvad\\\",\\\"icon\\\":\\\"/usr/share/icons/hicolor/scalable/apps/mullvad-browser.svg\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{\\\"command\\\":\\\"mullvad-browser\\\"},\\\"angle\\\":-1}",
                    ),
                    "Apply Browser info on dconf shell",
                );
            }
        }
        _ => log::info!("No browser setup selected!"),
    }
    println!();
    // Terminal configuration //
    log::info!("Installing terminal : {:?}", config.terminal);
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
        _ => log::info!("No terminal setup selected!"),
    }
    //////////
    files_eval(
        sed_file(
            "/mnt/usr/local/bin/shell-rocket",
            "gnome-terminal --",
            &(terminal_choice.clone()+" "+if terminal_choice == "gnome-terminal" { "--" } else { "-e" }),
        ),
        "Set terminal on shell rocket",
    );
    files_eval(
        sed_file(
            "/mnt/usr/share/applications/shell.desktop",
            "gnome-terminal",
            &terminal_choice,
        ),
        "Set terminal call on shell.desktop file",
    );
    if config.desktop.contains("gnome") {
        files_eval(
            sed_file(
                "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                "gnome-terminal",
                &terminal_choice,
            ),
            "Set terminal call on dconf file",
        );
    }
    //////////
    println!();
    log::info!("Installing timeshift : {}", config.timeshift);
    if config.timeshift {
        base::setup_timeshift();
    }
    println!();
    log::info!("Installing snapper : {}", config.snapper);
    if config.snapper {
        base::setup_snapper();
    }
    println!();
    log::info!("Installing flatpak : {}", config.flatpak);
    if config.flatpak {
        base::install_flatpak();
    }
    log::info!("Extra packages : {:?}", config.extra_packages);
    let mut extra_packages: Vec<&str> = Vec::new();
    for i in 0..config.extra_packages.len() {
        extra_packages.push(config.extra_packages[i].as_str());
    }
    install(PackageManager::Pacman, extra_packages);
    println!();
    log::info!("Enabling system services...");
    base::enable_system_services();
    println!("---------");
    for i in 0..config.users.len() {
        log::info!("Creating user : {}", config.users[i].name);
        //log::info!("Setting use password : {}", config.users[i].password);
        log::info!("Enabling root for user : {}", config.users[i].hasroot);
        log::info!("Setting user shell : {}", config.users[i].shell);

        match config.users[i].shell.to_lowercase().as_str() {
            "bash" => shells::install_shell_setup(ShellSetup::Bash),
            "fish" => shells::install_shell_setup(ShellSetup::Fish),
            "zsh" => shells::install_shell_setup(ShellSetup::Zsh),
            _ => log::info!("No shell setup selected!"),
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
    //log::info!("Setting root password : {}", config.rootpass);
    users::root_pass(config.rootpass.as_str());
    println!();
    log::info!("Installation log file copied to /var/log/aegis.log");
    files::copy_file("/tmp/aegis.log", "/mnt/var/log/aegis.log");
    println!("Installation finished! You may reboot now!")
}

fn disable_xsession(session: &str) {
    log::debug!("Disabling {}", session);
    files::rename_file(&("/mnt/usr/share/xsessions/".to_owned()+session), &("/mnt/usr/share/xsessions/".to_owned()+session+".disable"));
}

fn disable_wsession(session: &str) {
    log::debug!("Disabling {}", session);
    files::rename_file(&("/mnt/usr/share/wayland-sessions/".to_owned()+session), &("/mnt/usr/share/wayland-sessions/".to_owned()+session+".disable"));
}

fn lightdm_set_session(setdesktop: &str) {
    if setdesktop.contains("gnome") {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=gnome-xorg",
            ),
            "Apply GNOME User Session on LightDM",
        );
    }
    if setdesktop.contains("xfce") {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=xfce",
            ),
            "Apply Hyprland User Session on LightDM",
        );
    }
    if setdesktop == "hyprland" {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=hyprland",
            ),
            "Apply Hyprland User Session on LightDM",
        );
    }
}