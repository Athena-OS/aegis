use crate::functions::*;
use crate::internal::install::install;
use shared::args::{self, DesktopSetup, ThemeSetup, DMSetup, ShellSetup, BrowserSetup, TerminalSetup, PartitionMode};
use shared::{debug, error, info};
use shared::files;
use shared::partition;
use shared::returncode_eval::files_eval;
use shared::serde::{self, Deserialize, Serialize};
use shared::serde_json;
use shared::strings::crash;
use std::path::{PathBuf};
//use std::io::{self, BufRead, BufReader};
//use std::process::{Command, Stdio};


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
    design: String,
    displaymanager: String,
    browser: String,
    terminal: String,
    flatpak: bool,
    zramd: bool,
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
    base::install_nix_config();
    info!("Installing bootloader : {}", config.bootloader.r#type);
    info!("Installing bootloader to : {}", config.bootloader.location);
    if config.bootloader.r#type == "grub-efi" {
        base::install_bootloader_efi(PathBuf::from(config.bootloader.location));
    } else if config.bootloader.r#type == "grub-legacy" {
        base::install_bootloader_legacy(PathBuf::from(config.bootloader.location));
    }
    // Set locales at the beginning to prevent some warning messages about "Setting locale failed"
    info!("Adding Locales : {:?}", config.locale.locale);
    locale::set_locale(config.locale.locale.join(" "));
    info!("Using console keymap : {}", config.locale.virtkeymap);
    info!("Using x11 keymap : {}", config.locale.x11keymap);
    locale::set_keyboard(config.locale.virtkeymap.as_str(), config.locale.x11keymap.as_str());
    info!("Setting timezone : {}", config.locale.timezone);
    locale::set_timezone(config.locale.timezone.as_str());
    info!("Hostname : {}", config.networking.hostname);
    network::set_hostname(config.networking.hostname.as_str());
    info!("Enabling ipv6 : {}", config.networking.ipv6);
    if config.networking.ipv6 {
        network::enable_ipv6();
    }
    info!("Enabling zramd : {}", config.zramd);
    if config.zramd {
        base::install_zram();
    }
    info!("Installing desktop : {:?}", config.desktop);
    match config.desktop.to_lowercase().as_str() {
        "gnome" => { //Note that the value on this match statement must fit the name in desktops.py of aegis-gui (then they are lowercase transformed)
            desktops::install_desktop_setup(DesktopSetup::Gnome);
        },
        "cinnamon" => desktops::install_desktop_setup(DesktopSetup::Cinnamon),
        "mate" => desktops::install_desktop_setup(DesktopSetup::Mate),
        "xfce refined" => desktops::install_desktop_setup(DesktopSetup::XfceRefined),
        "xfce picom" => desktops::install_desktop_setup(DesktopSetup::XfcePicom),
        "none" => desktops::install_desktop_setup(DesktopSetup::None),
        _ => info!("No desktop setup selected!"),
    }
    info!("Installing design : {:?}", config.design);

    match config.design.to_lowercase().as_str() {
        "cyborg" => themes::install_theme_setup(ThemeSetup::Cyborg),
        "graphite" => themes::install_theme_setup(ThemeSetup::Graphite),
        "hackthebox" => themes::install_theme_setup(ThemeSetup::HackTheBox), //Note that the value on this match statement must fit the name in themes.py of aegis-gui (then they are lowercase transformed)
        "redmoon" => themes::install_theme_setup(ThemeSetup::RedMoon),
        "samurai" => themes::install_theme_setup(ThemeSetup::Samurai),
        "sweet" => themes::install_theme_setup(ThemeSetup::Sweet),
        "temple" => themes::install_theme_setup(ThemeSetup::Temple),
        _ => info!("No design setup selected!"),
    }
    info!("Installing display manager : {:?}", config.displaymanager);
    match config.displaymanager.to_lowercase().as_str() {
        "gdm" => {
            displaymanagers::install_dm_setup(DMSetup::Gdm);
        },
        "lightdm neon" => {
            displaymanagers::install_dm_setup(DMSetup::LightDMNeon);
        },
        _ => info!("No display manager setup selected!"),
    }
    info!("Installing browser : {:?}", config.browser);
    /*if let Some(browser) = &config.browser {
        browsers::install_browser_setup(*browser);
    }*/
    match config.browser.to_lowercase().as_str() {
        "firefox" => {
            browsers::install_browser_setup(BrowserSetup::Firefox);
        },
        _ => info!("No browser setup selected!"),
    }
    // Terminal configuration //
    info!("Installing terminal : {:?}", config.terminal);
    match config.terminal.to_lowercase().as_str() {
        "alacritty" => {
            terminals::install_terminal_setup(TerminalSetup::Alacritty);
        },
        "kitty" => {
            terminals::install_terminal_setup(TerminalSetup::Kitty);
        },
        _ => info!("No terminal setup selected!"),
    }
    // Misc Settings
    info!("Installing flatpak : {}", config.flatpak);
    if config.flatpak {
        base::install_flatpak();
    }
    // Users
    for i in 0..config.users.len() {
        info!("Creating user : {}", config.users[i].name);
        //info!("Setting use password : {}", config.users[i].password);
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
            config.users[i].password.as_str(),
            false,
        );
    }
    //info!("Setting root password : {}", config.rootpass);
    users::root_pass(config.rootpass.as_str());
    info!("Install Athena OS");
    let exit_code = install();
    files::copy_multiple_files("/etc/NetworkManager/system-connections/*", "/mnt/etc/NetworkManager/system-connections/");
    info!("Installation log file copied to /var/log/aegis.log");
    files_eval(files::create_directory("/mnt/var/log"), "create /mnt/var/log");
    files::copy_file("/tmp/aegis.log", "/mnt/var/log/aegis.log");
    if config.bootloader.r#type == "grub-efi" {
        partition::umount("/mnt/boot/efi");
    }
    else {
        partition::umount("/mnt/boot");
    }
    partition::umount("/mnt/home");
    partition::umount("/mnt");
    if exit_code == 0 {
        info!("Installation finished! You may reboot now!");
    }
    else {
        error!("Installation failed. Exit code: {}", exit_code);
        /*
        // The following code should be removed. The log generation must be proposed by Aegis TUI and GUI
        if prompt_user_for_logs() {
            info!("Generating log URL...");
            run_logs_command();
        }
        */
    }
    
    exit_code
}

/*
// Prompt the user to generate logs and return true if the answer is 'Y'
fn prompt_user_for_logs() -> bool {
    info!("\nDo you want to generate logs of the failed install to communicate to the team? (Y/n)");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read user input");

    // If the input is empty, set the default choice to 'Y'
    let choice = if input.trim().is_empty() { "Y".to_lowercase() } else { input.trim().to_lowercase() };

    // Check if the choice is 'y'
    choice == "y"
}

// Run the command to send logs to termbin.com
fn run_logs_command() {
    // Create a new command to run the specified shell command
    let mut logs_command = Command::new("sh")
        .args(["-c", "cat /tmp/aegis.log | nc termbin.com 9999"])
        .stdout(Stdio::piped())  // Redirect standard output to a pipe
        .stderr(Stdio::piped())  // Redirect standard error to a pipe
        .spawn()  // Start the command as a new process
        .expect("Failed to start logs command.");  // Handle any errors during command startup

    let stdout_handle = logs_command.stdout.take().expect("Failed to open stdout pipe.");
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout_handle);
        for line in reader.lines().map_while(Result::ok) {
            info!("{}", line);
        }
    });

    let stderr_handle = logs_command.stderr.take().expect("Failed to open stderr pipe.");
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr_handle);
        for line in reader.lines().map_while(Result::ok) {
            error!("{}", line);
        }
    });

    // Wait for the logs command to complete and log its exit status
    let logs_status = logs_command.wait();
    match logs_status {
        Ok(exit_status) => match exit_status.code() {
            Some(code) => {
                if code == 0 {
                    info!("Log URL generation completed.");
                } else {
                    error!("Error on generating log URL. Exit code: {}", code);
                }
            }
            None => info!("Logs command terminated without an exit code."),
        },
        Err(err) => error!("Failed to wait for logs command: {}", err),
    }

    // Wait for the threads capturing output to finish before returning
    stdout_thread.join().expect("Failed to join stdout thread.");
    stderr_thread.join().expect("Failed to join stderr thread.");
}
*/