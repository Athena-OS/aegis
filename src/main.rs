mod args;
mod functions;
mod internal;
mod logging;

use crate::args::{BootloaderSubcommand, Command, Cli, UsersSubcommand};
use crate::functions::*;
use clap::Parser;

fn main() {
    human_panic::setup_panic!();
    let cli = Cli::parse();
    println!("verbose: {}", cli.verbose);
    let log_file_path = "/tmp/athena-install";
    logging::init(cli.verbose, log_file_path);
    match cli.command {
        Command::Partition(args) => {
            let mut partitions = args.partitions;
            partition::partition(
                args.device,
                args.mode,
                args.efi,
                &mut partitions,
            );
        }
        Command::InstallBase => {
            base::install_base_packages();
        }
        Command::Locale(args) => {
            locale::set_locale(args.locales.join(" "));
            locale::set_keyboard(&args.keyboard);
            locale::set_timezone(&args.timezone);
        }
        Command::InstallPackages(args) => {
            base::install_packages(args.kernel);
        }
        Command::GenFstab => {
            base::genfstab();
        }
        Command::SetupTimeshift => base::setup_timeshift(),
        Command::SetupSnapper => base::setup_snapper(),
        Command::Bootloader { subcommand } => match subcommand {
            BootloaderSubcommand::GrubEfi { efidir } => {
                base::install_bootloader_efi(efidir);
            }
            BootloaderSubcommand::GrubLegacy { device } => {
                base::install_bootloader_legacy(device);
            }
        }
        Command::Networking(args) => {
            if args.ipv6 {
                network::create_hosts();
                network::enable_ipv6()
            } else {
                network::create_hosts();
            }
            network::set_hostname(&args.hostname);
        }
        Command::Zram => {
            base::install_zram();
        }
        Command::Users { subcommand } => match subcommand {
            UsersSubcommand::NewUser(args) => {
                users::new_user(
                    &args.username,
                    args.hasroot,
                    &args.password,
                    true,
                    &args.shell,
                );
            }
            UsersSubcommand::RootPass { password } => {
                users::root_pass(&password);
            }
        },
        Command::Nix => {
            base::install_homemgr();
        }
        Command::Flatpak => {
            base::install_flatpak();
        }
        Command::Cuda => {
            base::install_cuda();
        }
        Command::Spotify => {
            base::install_spotify();
        }
        Command::CherryTree => {
            base::install_cherrytree();
        }
        Command::Flameshot => {
            base::install_flameshot();
        }
        Command::BusyBox => {
            base::install_busybox();
        }
        Command::Toybox => {
            base::install_toybox();
        }
        Command::Config { config } => {
            crate::internal::config::read_config(config);
        }
        Command::Desktops { desktop } => {
            desktops::install_desktop_setup(desktop);
        }
        Command::Themes { theme } => {
            themes::install_theme_setup(theme);
        }
        Command::DisplayManagers { displaymanager } => {
            displaymanagers::install_dm_setup(displaymanager);
        }
        Command::Shells { shell } => {
            shells::install_shell_setup(shell);
        }
        Command::Browsers { browser } => {
            browsers::install_browser_setup(browser);
        }
        Command::Terminals { terminal } => {
            terminals::install_terminal_setup(terminal);
        }
        Command::EnableServices => {
            base::enable_system_services();
        }
    }
}
