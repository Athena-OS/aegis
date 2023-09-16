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

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Partition the install destination
    #[command(name = "partition")]
    Partition(PartitionArgs),

    /// Install base packages, optionally define a different kernel
    #[command(name = "install-base")]
    InstallBase(InstallBaseArgs),

    /// Generate fstab file for mounting partitions
    #[command(name = "genfstab")]
    GenFstab,

    /// Setup Timeshift
    #[command(name = "setup-timeshift")]
    SetupTimeshift,

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

    /// Configure users and passwords
    #[command(name = "users")]
    Users {
        #[command(subcommand)]
        subcommand: UsersSubcommand,
    },

    /// Install the Nix package manager
    #[command(name = "nix")]
    Nix,

    /// Install Flatpak and enable FlatHub
    #[command(name = "flatpak")]
    Flatpak,

    /// Read Jade installation config
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
}

#[derive(Debug, Args)]
pub struct PartitionArgs {
    /// If jade should automatically partition (mode = auto)
    /// or the user manually partitioned it (mode = manual)
    #[arg(value_enum)]
    pub mode: PartitionMode,

    /// The device to partition
    #[arg(required_if_eq("mode", "PartitionMode::Auto"), required = false)]
    pub device: PathBuf,

    /// If the install destination should be partitioned with EFI
    #[arg(long)]
    pub efi: bool,

    #[arg(long)]
    pub unakite: bool,

    /// The partitions to use for manual partitioning
    #[arg(required_if_eq("mode", "PartitionMode::Manual"), value_parser = parse_partitions)]
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Args)]
pub struct InstallBaseArgs {
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
    pub keyboard: String,

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
        /// The password to set. NOTE: Takes hashed password, use `openssl passwd -1 <password>` to generate the hash.
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

    #[value(name = "xfce")]
    Xfce,

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

    #[value(name = "None/DIY")]
    None,
}
