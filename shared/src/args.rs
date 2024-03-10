use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "aegis-installer")]
#[command(author=env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum PackageManager {
    #[value(name = "pacman")]
    Pacman,

    #[value(name = "pacstrap")]
    Pacstrap,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Partition the install destination
    #[command(name = "partition")]
    Partition(PartitionArgs),

    /// Install base packages
    #[command(name = "install-base")]
    InstallBase,

    /// Install packages, optionally define a different kernel
    #[command(name = "install-packages")]
    InstallPackages(InstallPackagesArgs),

    /// Generate fstab file for mounting partitions
    #[command(name = "genfstab")]
    GenFstab,
    
    /// Setup Snapper
    #[command(name = "setup-snapper")]
    SetupSnapper,

    /// Install the bootloader
    #[command(name = "bootloader")]
    Bootloader {
        #[clap(subcommand)]
        subcommand: BootloaderSubcommand,
    },

    /// Set locale
    #[command(name = "locale")]
    Locale(LocaleArgs),

    /// Set up networking
    #[command(name = "networking")]
    Networking(NetworkingArgs),

    /// Set up zramd
    #[command(name = "zramd")]
    Zram,

    /// Install Flatpak and enable FlatHub
    #[command(name = "flatpak")]
    Flatpak,

    /// Set up hardened
    #[command(name = "hardened")]
    Hardened,

    /// Configure users and passwords
    #[command(name = "users")]
    Users {
        #[command(subcommand)]
        subcommand: UsersSubcommand,
    },

    /// Set install parameters
    #[command(name = "params")]
    InstallParams(InstallArgs),

    /// Install CUDA
    #[command(name = "cuda")]
    Cuda,

    /// Install Spotify
    #[command(name = "spotify")]
    Spotify,

    /// Install CherryTree
    #[command(name = "cherrytree")]
    CherryTree,

    /// Install Flameshot
    #[command(name = "flameshot")]
    Flameshot,

    /// Install BusyBox
    #[command(name = "busybox")]
    BusyBox,

    /// Install Toybox
    #[command(name = "toybox")]
    Toybox,

    /// Read Aegis installation config
    #[command(name = "config")]
    Config {
        /// The config file to read
        config: PathBuf,
    },

    /// Install a graphical desktop
    #[command(name = "desktops")]
    Desktops {
        /// The desktop setup to use
        #[arg(value_enum)]
        desktop: DesktopSetup,
    },

    /// Install a graphical theme
    #[command(name = "themes")]
    Themes {
        /// The theme setup to use
        #[arg(value_enum)]
        theme: ThemeSetup,
    },

    /// Install a display manager
    #[command(name = "displaymanagers")]
    DisplayManagers {
        /// The display manager setup to use
        #[arg(value_enum)]
        displaymanager: DMSetup,
    },

    /// Install a shell
    #[command(name = "shells")]
    Shells {
        /// The shell setup to use
        #[arg(value_enum)]
        shell: ShellSetup,
    },

    /// Install a browser
    #[command(name = "browsers")]
    Browsers {
        /// The browser setup to use
        #[arg(value_enum)]
        browser: BrowserSetup,
    },

    /// Install a terminal
    #[command(name = "terminals")]
    Terminals {
        /// The terminal setup to use
        #[arg(value_enum)]
        terminal: TerminalSetup,
    },

    /// Enable services
    #[command(name = "enable-services")]
    EnableServices,
}

#[derive(Debug, Args)]
pub struct PartitionArgs {
    /// If aegis should automatically partition (mode = auto)
    /// or the user manually partitioned it (mode = manual)
    #[arg(value_enum)]
    pub mode: PartitionMode,

    /// The device to partition
    #[arg(required_if_eq("mode", "PartitionMode::Auto"), required = false)]
    pub device: PathBuf,

    /// If the install destination should be partitioned with EFI
    #[arg(long)]
    pub efi: bool,

    /// If the install destination should have Swap partition
    #[arg(long)]
    pub swap: bool,

    /// Swap partition size
    #[arg(long)]
    pub swap_size: String,

    /// The partitions to use for manual partitioning
    #[arg(required_if_eq("mode", "PartitionMode::Manual"), value_parser = parse_partitions)]
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Args)]
pub struct InstallPackagesArgs {
    #[clap(long)]
    pub kernel: String,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub mountpoint: String,
    pub blockdevice: String,
    pub filesystem: String,
}

impl Partition {
    pub fn new(mountpoint: String, blockdevice: String, filesystem: String) -> Self {
        Self {
            mountpoint,
            blockdevice,
            filesystem,
        }
    }
}

pub fn parse_partitions(s: &str) -> Result<Partition, &'static str> { // to rewrite
    println!("{}", s);
    Ok(Partition::new(
        s.split(':').collect::<Vec<&str>>()[0].to_string(),
        s.split(':').collect::<Vec<&str>>()[1].to_string(),
        s.split(':').collect::<Vec<&str>>()[2].to_string(),
    ))
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum PartitionMode {
    #[value(name = "auto")]
    Auto,
    #[value(name = "manual")]
    Manual,
}

#[derive(Debug, Subcommand)]
pub enum BootloaderSubcommand {
    /// Install GRUB in EFI mode
    #[clap(name = "grub-efi")]
    GrubEfi {
        /// The directory to install the EFI bootloader to
        efidir: PathBuf,
    },

    /// Install GRUB in legacy (BIOS) mode
    #[clap(name = "grub-legacy")]
    GrubLegacy {
        /// The device to install the bootloader to
        device: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct LocaleArgs {
    /// The keyboard layout to use
    pub virtkeyboard: String,
    pub x11keyboard: String,

    /// The timezone to use
    pub timezone: String,

    /// The locales to set
    pub locales: Vec<String>,
}

#[derive(Debug, Args)]
pub struct NetworkingArgs {
    /// The hostname to assign to the system
    pub hostname: String,

    /// Whether IPv6 loopback should be enabled
    #[arg(long)]
    pub ipv6: bool,
}

#[derive(Debug, Subcommand)]
pub enum UsersSubcommand {
    /// Create a new user
    #[command(name="new-user", aliases=&["newUser"])]
    NewUser(NewUserArgs),

    /// Set the password of the root user
    #[command(name="root-password", aliases=&["root-pass", "rootPass"])]
    RootPass {
        /// The password to set. NOTE: Takes hashed password, use `openssl passwd -6 <password>` to generate the hash.
        password: String,
    },
}

#[derive(Debug, Args)]
pub struct NewUserArgs {
    /// The name of the user to create
    pub username: String,

    /// If the user should have root privileges
    #[arg(long, aliases=&["has-root", "sudoer", "root"])]
    pub hasroot: bool,

    /// The password to set. NOTE: Takes hashed password, use `openssl passwd -6 <password>` to generate the hash.
    /// When not providing a password openssl jumps into an interactive masked input mode allowing you to hide your password
    /// from the terminal history.
    pub password: String,

    /// The shell to use for the user. The current options are bash, csh, fish, tcsh, and zsh.
    /// If a shell is not specified or unknown, it defaults to fish.
    pub shell: String,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// The number of cores to use
    pub cores: String,

    /// The number of jobs to use
    pub jobs: String,

    /// Keep the install if a build fails
    pub keep: bool,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum DesktopSetup {
    #[value(name = "onyx")]
    Onyx,

    #[value(name = "gnome")]
    Gnome,

    #[value(name = "kde", aliases = ["plasma"])]
    Kde,

    #[value(name = "budgie")]
    Budgie,

    #[value(name = "cinnamon")]
    Cinnamon,

    #[value(name = "mate")]
    Mate,

    #[value(name = "xfce-refined")]
    XfceRefined,

    #[value(name = "xfce-picom")]
    XfcePicom,

    #[value(name = "enlightenment")]
    Enlightenment,

    #[value(name = "lxqt")]
    Lxqt,

    #[value(name = "sway")]
    Sway,

    #[value(name = "i3")]
    I3,

    #[value(name = "herbstluftwm")]
    Herbstluftwm,

    #[value(name = "awesome")]
    Awesome,

    #[value(name = "bspwm")]
    Bspwm,

    #[value(name = "hyprland")]
    Hyprland,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum ThemeSetup {
    #[value(name = "akame")]
    Akame,

    #[value(name = "cyborg")]
    Cyborg,

    #[value(name = "graphite")]
    Graphite,

    #[value(name = "hackthebox")]
    HackTheBox,

    #[value(name = "samurai")]
    Samurai,

    #[value(name = "sweet")]
    Sweet,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum DMSetup {
    #[value(name = "gdm")]
    Gdm,

    #[value(name = "lightdm-neon")]
    LightDMNeon,

    #[value(name = "sddm")]
    Sddm,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShellSetup {
    #[value(name = "bash")]
    Bash,

    #[value(name = "fish")]
    Fish,

    #[value(name = "zsh")]
    Zsh,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum BrowserSetup {
    #[value(name = "firefox")]
    Firefox,

    #[value(name = "brave")]
    Brave,

    #[value(name = "None/DIY")]
    None,
}

#[derive(Debug, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum TerminalSetup {
    #[value(name = "alacritty")]
    Alacritty,

    #[value(name = "cool-retro-term")]
    CoolRetroTerm,

    #[value(name = "foot")]
    Foot,

    #[value(name = "gnome-terminal")]
    GnomeTerminal,

    #[value(name = "kitty")]
    Kitty,

    #[value(name = "konsole")]
    Konsole,

    #[value(name = "terminator")]
    Terminator,

    #[value(name = "terminology")]
    Terminology,

    #[value(name = "urxvt")]
    Urxvt,

    #[value(name = "xfce4-terminal")]
    Xfce,

    #[value(name = "xterm")]
    Xterm,

    #[value(name = "None/DIY")]
    None,
}