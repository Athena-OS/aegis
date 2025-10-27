use crate::{
    functions::*,
    internal::{install::install, packages, secure},
};
use log::{debug, error, info};
use serde_json::{self, Value, Map};
use shared::{
    args::{self, Config, ConfigInput, DesktopSetup, ExtendIntoString, PackageManager, ThemeSetup, DMSetup, ShellSetup, get_fedora_version, set_base, is_arch, is_fedora, is_nix},
    files,
    partition,
    returncode_eval::files_eval,
    strings::crash,
};
use std::{
    fs, path::{Path, PathBuf},
};

fn merge_values(dst: &mut Value, src: Value) {
    match (dst, src) {
        (Value::Object(a), Value::Object(b)) => {
            for (k, v) in b {
                merge_values(a.entry(k).or_insert(Value::Null), v);
            }
        }
        // Replace arrays by default (predictable). If you'd rather append, change this arm.
        (dst_slot @ Value::Array(_), Value::Array(b)) => {
            *dst_slot = Value::Array(b);
        }
        // For all other cases, override.
        (dst_slot, v) => {
            *dst_slot = v;
        }
    }
}

// Parse a string that may be:
//  - a single JSON object
//  - a JSON array of objects
//  - NDJSON (one object per line)
// Returns a list of JSON Values to be merged in order.
fn parse_fragments_from_str(label: &str, s: &str) -> Vec<Value> {
    // Try as one JSON value first
    if let Ok(val) = serde_json::from_str::<Value>(s) {
        match val {
            Value::Array(arr) => arr,
            other => vec![other],
        }
    } else {
        // Fallback: NDJSON by lines
        let mut out = Vec::new();
        for (i, line) in s.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }
            match serde_json::from_str::<Value>(line) {
                Ok(v) => out.push(v),
                Err(e) => {
                    crash(
                        format!("Parse JSON (NDJSON) from {label} line {} ERROR: {e}", i + 1),
                        1,
                    );
                }
            }
        }
        if out.is_empty() {
            crash(format!("No valid JSON found in {label}"), 1);
        }
        out
    }
}

fn unwrap_known_roots(root: &mut serde_json::Value) {
    // snapshot the wrapped objects (if any) without holding the borrow
    let (cfg, drv) = match root {
        serde_json::Value::Object(map) => (map.get("config").cloned(), map.get("drives").cloned()),
        _ => (None, None),
    };

    // merge their contents into the root
    if let Some(c) = cfg { merge_values(root, c); }
    if let Some(d) = drv { merge_values(root, d); }

    // drop the wrapper keys to avoid surprises
    if let serde_json::Value::Object(map) = root {
        map.remove("config");
        map.remove("drives");
    }
}

pub fn read_config(inputs: &[ConfigInput]) -> Config {
    let mut merged = Value::Object(Map::new());
    
    for (idx, input) in inputs.iter().enumerate() {
        match input {
            ConfigInput::File(path) => {
                // Read original content
                let data = fs::read_to_string(path).unwrap_or_else(|e| {
                    crash(format!("Read config file {path:?}  ERROR: {e}"), e.raw_os_error().unwrap_or(1))
                });

                // Redact on disk for logging (keeps your existing behavior)
                // NOTE: This modifies the file(s). If you do not want that, remove these calls
                // and just log a redacted copy from memory (as done for JSON strings).
                files_eval(
                    files::sed_file(
                        path.to_str().unwrap(),
                        "\"password_hash\":.*",
                        "\"password_hash\": \"*REDACTED*\",",
                    ),
                    "Redact user information",
                );
                files_eval(
                    files::sed_file(
                        path.to_str().unwrap(),
                        "\"root_passwd_hash\":.*",
                        "\"root_passwd_hash\": \"*REDACTED*\",",
                    ),
                    "Redact root information",
                );

                // Log redacted on-disk content
                let redacted = fs::read_to_string(path)
                    .expect("Failed to read config file after redaction");
                info!("Configuration fragment #{idx} (file: {}):\n{redacted}", path.display());

                // Merge original (unredacted) content
                let frags = parse_fragments_from_str(&format!("file {}", path.display()), &data);
                for frag in frags {
                    merge_values(&mut merged, frag);
                }

                debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parsed and merged config file {path:?}");
            }

            ConfigInput::JsonString(s) => {
                info!("Configuration fragment #{idx} (json/string or stdin):\n{s}");

                // Merge original
                let frags = parse_fragments_from_str("JSON string/STDIN", s);
                for frag in frags {
                    merge_values(&mut merged, frag);
                }

                debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parsed and merged config from JSON string/STDIN");
            }
        }
    }

    unwrap_known_roots(&mut merged);

    // Deserialize into your strong `Config` type
    match serde_json::from_value::<Config>(merged) {
        Ok(cfg) => {
            debug!("[ \x1b[2;1;32mOK\x1b[0m ] Deserialized merged configuration into Config");
            cfg
        }
        Err(e) => {
            crash(format!("Merged configuration is invalid for Config  ERROR: {e}"), 1);
        }
    }
}

pub fn install_config(inputs: &[ConfigInput], log_path: String) -> i32 {
    let config = read_config(inputs);
    set_base(&config.base);
    let mut exit_code = 0;
    let kernel = "linux-lts";
    let mut package_set = packages::to_strings(packages::COMMON);
    
    if is_arch() {
        packages::extend(&mut package_set, packages::ARCH_ONLY);
    } else if is_fedora() {
        let fedora_version = get_fedora_version();

        package_set.push(format!("https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-{fedora_version}.noarch.rpm"));
        package_set.push(format!("https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-{fedora_version}.noarch.rpm"));

        /*****TEMPORARY DISABLE SELINUX*****/
        if secure::selinux_enabled() {
            secure::set_selinux_mode("0");
        }

        // Then append Fedora-only package names
        packages::extend(&mut package_set, packages::FEDORA_ONLY);
    };

    /*    PARTITIONING    */
    let disk_type = config.partition.content.table_type;
    info!("Disk device to use : {}", config.partition.device);
    info!("Partition Table type : {disk_type}");

    // Build args::Partition list from the structured JSON.
    // Encrypt per-partition if flags contain "encrypt".
    let mut partitions: Vec<args::Partition> = Vec::new();
    for p in &config.partition.content.partitions {
        let action = p.action.clone(); // e.g. create, modify, delete
        let mountpoint    = p.mountpoint.clone(); // empty for swap
        let blockdevice   = p.blockdevice.clone();                         // "/dev/nvme0n1p2"
        let start   = p.start.clone();                         // start sector
        let end   = p.end.clone();                         // end sector
        let filesystem    = p.filesystem.clone();      // "ext4", "vfat", "swap"
        let flags         = p.flags.clone();

        partitions.push(args::Partition::new(
            action, mountpoint, blockdevice, start, end, filesystem, flags,
        ));
    }

    // Handle both "/dev/nvme0n1" and "nvme0n1"
    let device = if config.partition.device.starts_with("/dev/") {
        PathBuf::from(&config.partition.device)
    } else {
        PathBuf::from("/dev").join(&config.partition.device)
    };
    
    partition::partition(
        device,
        &disk_type,
        &mut partitions,
    );

    if is_nix() {
        base::install_nix_config();
    }
    /* BOOTLOADER SET */

    if partition::is_uefi() {
        package_set.push("efibootmgr".into());
        package_set.push("athena-secureboot".into());
        package_set.push("mokutil".into());
        package_set.push("sbsigntools".into());
        package_set.push("shim-signed".into());

        if is_fedora() {
            package_set.push("grub2-efi".into());
            package_set.push("grub2-efi-x64-modules".into()); // Not sure if it works also for ARM CPU
            package_set.push("grubby".into());
            package_set.push("shim-*".into());
            let grub_cfg_path = "/mnt/boot/efi/EFI/fedora/grub.cfg";
            let path = Path::new(grub_cfg_path);
            if path.exists() && path.is_file() {
                files::remove_file(grub_cfg_path);
            }
        }
    }
    partition::partition_info();
    /**************************/
    /*        DESKTOP         */
    info!("Selected desktop : {:?}", config.desktop);
    /*if let Some(desktop) = &config.desktop {
        desktops::install_desktop_setup(*desktop);
    }*/
    match config.desktop.to_lowercase().as_str() {
        "onyx" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Onyx)),
        "kde plasma" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Kde)), //Note that the value on this match statement must fit the name in desktops.py of aegis-gui (then they are lowercase transformed)
        "mate" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Mate)),
        "gnome" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Gnome)),
        "cinnamon" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Cinnamon)),
        "xfce picom" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::XfcePicom)),
        "xfce" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::XfceRefined)),
        "budgie" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Budgie)),
        "enlightenment" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Enlightenment)),
        "lxqt" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Lxqt)),
        "sway" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Sway)),
        "i3" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::I3)),
        "herbstluftwm" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Herbstluftwm)),
        "awesome" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Awesome)),
        "bspwm" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Bspwm)),
        "hyprland" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::Hyprland)),
        "none" => package_set.extend_into(desktops::install_desktop_setup(DesktopSetup::None)),
        _ => info!("No desktop setup selected!"),
    }
    /**************************/

    /*     DISPLAY MANAGER    */
    info!("Selected display manager : {:?}", config.displaymanager);
    package_set.extend_into(displaymanagers::install_dm_setup(DMSetup::Sddm));
    //match config.displaymanager.to_lowercase().as_str() {
    //    "gdm" => package_set.extend_into(displaymanagers::install_dm_setup(DMSetup::Gdm)),
    //    "lightdm neon" => package_set.extend_into(displaymanagers::install_dm_setup(DMSetup::LightDMNeon)),
    //    "sddm" => package_set.extend_into(displaymanagers::install_dm_setup(DMSetup::Sddm)),
    //    _ => info!("No display manager setup selected!"),
    //}
    /**************************/
    /*         DESIGN         */
    info!("Selected design : {:?}", config.design);
    match config.design.to_lowercase().as_str() {
        "cyborg" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::Cyborg)),
        "graphite" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::Graphite)),
        "hackthebox" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::HackTheBox)), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "redmoon" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::RedMoon)),
        "samurai" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::Samurai)),
        "sweet" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::Sweet)),
        "temple" => package_set.extend_into(themes::install_theme_setup(ThemeSetup::Temple)),
        _ => info!("No design setup selected!"),
    }
    /**************************/
    /*    EXTRA PACKAGES    */
    info!("Extra packages : {:?}", config.extra_packages);
    package_set.extend(config.extra_packages.clone());
    /**************************/
    /*          MISC         */
    info!("Selecting zramd.");
    package_set.push("zram-generator".into());
    /**************************/
    /*         USERS         */
    for i in 0..config.users.len() {
        match config.users[i].shell.to_lowercase().as_str() {
            "bash" => package_set.extend_into(shells::install_shell_setup(ShellSetup::Bash)),
            "fish" => package_set.extend_into(shells::install_shell_setup(ShellSetup::Fish)),
            "zsh" => package_set.extend_into(shells::install_shell_setup(ShellSetup::Zsh)),
            _ => info!("No shell setup selected!"),
        }
    }
    /**************************/
    
    /********** INSTALLATION **********/
    if !is_nix() {
        package_set.sort();
        package_set.dedup();
        exit_code = base::install_packages(package_set, kernel);
        base::genfstab();
    }

    /********** CONFIGURATION **********/
    /*         LOCALES        */
    // Set locales at the beginning to prevent some warning messages about "Setting locale failed"
    info!("Adding Locale : {}", config.locale);
    locale::set_locale(config.locale.clone());
    let chosen_kbd = config.keyboard_layout.as_deref().unwrap_or("us");
    info!("Using keymap : {chosen_kbd}");
    if let Err(e) = locale::set_keyboard(chosen_kbd) {
        error!("Error setting keyboard configuration: {e}");
    }
    info!("Setting timezone : {}", config.timezone);
    locale::set_timezone(config.timezone.as_str());
    /**************************/
    if is_arch() {
        info!("Processing all presets.");
        base::preset_process();
    }
    info!("Hostname : {}", config.hostname);
    network::set_hostname(config.hostname.as_str());

    /**************************/
    if !is_nix() {
        network::create_hosts();
        /**************************/
        /*     DESKTOP CONFIG     */
        info!("Configuring desktop : {:?}", config.desktop);
        match config.desktop.to_lowercase().as_str() {
            "gnome" => desktops::configure_gnome(),
            "cinnamon" => desktops::configure_cinnamon(),
            "hyprland" => desktops::configure_hyprland(),
            "xfce" => desktops::configure_xfce(),
            _ => info!("No desktop configuration needed."),
        }
        /**************************/
        /*      DESIGN CONFIG     */
        info!("Configuring design : {:?}", config.design);
        match config.design.to_lowercase().as_str() {
            "cyborg" => themes::configure_cyborg(),
            "graphite" => themes::configure_graphite(),
            "hackthebox" => themes::configure_hackthebox(),
            "redmoon" => themes::configure_redmoon(),
            "samurai" => themes::configure_samurai(),
            "sweet" => themes::configure_sweet(),
            "temple" => themes::configure_temple(),
            _ => info!("No design configuration needed."),
        }
        /**************************/

        /*info!("Installing snapper : {}", config.snapper);
        if config.snapper {
            base::setup_snapper();
        }*/

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
        info!("Enabling zramd.");
        base::configure_zram();

        /*info!("Hardening system : {}", config.hardened);
        if config.hardened {
            secure::secure_password_config();
            secure::secure_ssh_config();
        }*/
        /**************************/
    }

    /* DISPLAY MANAGER CONFIG */
    info!("Configuring display manager : {:?}", config.displaymanager);
    displaymanagers::configure_sddm();
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => displaymanagers::configure_gdm(&config.desktop),
        "lightdm neon" => displaymanagers::configure_lightdm_neon(&config.desktop),
        "sddm" => displaymanagers::configure_sddm(), // In Fedora must be AFTER GNOME configuration to enable SDDM correctly, because GDM must be first disabled
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

    /*      USER CONFIG      */
    for i in 0..config.users.len() {
        info!("Creating user : {}", config.users[i].name);
        //info!("Setting user password : {}", config.users[i].password.as_str());
        info!("Setting user shell : {}", config.users[i].shell);

        users::new_user(
            config.users[i].name.as_str(),
            config.users[i].password.as_str(),
            &config.users[i].groups,
            "bash", //config.users[i].shell.as_str(), // Use bash because it must be the shell associated to the user in order to source the initial .sh files at login time
        );
    }
    //info!("Setting root password : {}", config.rootpass.as_str());
    users::root_pass(config.rootpass.as_str());

    if is_nix() {
        info!("Install Athena OS");
        exit_code = install(PackageManager::Nix, vec![], None);
    }
    
    /*    BOOTLOADER CONFIG     */
    // After root creation because mokutil needs root psw to import certificate
    if partition::is_uefi() {
        base::configure_bootloader_efi(PathBuf::from("/boot/efi"), kernel);
    } else {
        base::configure_bootloader_legacy(PathBuf::from(config.partition.device));
    }
    /**************************/

    /**************************/
    if !is_nix() {
        /*    ENABLE SERVICES    */
        info!("Enabling system services...");
        base::enable_system_services();
    }
    /**************************/
    /*   SET SELINUX CONTEXT   */
    if is_fedora() {
        info!("Applying security labels on files...");
        secure::set_security_context();
    }
    /**************************/
    files::copy_multiple_files("/etc/NetworkManager/system-connections/*", "/mnt/etc/NetworkManager/system-connections/");
    info!("Installation log file copied to /var/log/aegis.log");
    files_eval(files::create_directory("/mnt/var/log"), "create /mnt/var/log");
    files::copy_file(&log_path, "/mnt/var/log/aegis.log");

    if is_fedora() && secure::selinux_enabled() {
        secure::set_selinux_mode("1");
    }

    for p in &config.partition.content.partitions {
        if p.flags.iter().any(|f| f.eq_ignore_ascii_case("encrypt")) {
            // p.blockdevice is the *underlying* partition (e.g., /dev/vda2)
            shared::partition::close_luks_best_effort(&p.blockdevice);
        }
    }

    partition::umount("/mnt"); // Recursive umount

    if exit_code == 0 {
        info!("Installation finished! You may reboot now!");
    }
    else {
        error!("Installation failed. Exit code: {exit_code}");
    }
    
    exit_code
}
