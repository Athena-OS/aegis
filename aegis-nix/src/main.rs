mod functions;
mod internal;
use crate::functions::*;
use shared::args::{BootloaderSubcommand, Command, Cli, UsersSubcommand};
use shared::clap::Parser;
use shared::human_panic;
use shared::logging;
use shared::partition;

fn main() {
    human_panic::setup_panic!();
    let cli = Cli::parse();
    println!("verbose: {}", cli.verbose);
    let log_file_path = "/tmp/aegis";
    logging::init(cli.verbose, log_file_path);
    // menu choice
    match cli.command {
        Command::Partition(args) => {
            let mut partitions = args.partitions;
            partition::partition(
                args.device,
                args.mode,
                args.efi,
                args.swap,
                args.swap_size,
                &mut partitions,
            );
        }
        Command::InstallBase => {
            base::install_nix_config();
        }
        Command::Bootloader { subcommand } => match subcommand {
            BootloaderSubcommand::GrubEfi { efidir } => {
                base::install_bootloader_efi(efidir);
            }
            BootloaderSubcommand::GrubLegacy { device } => {
                base::install_bootloader_legacy(device);
            }
        }
        Command::Locale(args) => {
            locale::set_locale(args.locales.join(" ")); // locale.gen file comes grom glibc package that is in base group package
            locale::set_keyboard(&args.virtkeyboard, &args.x11keyboard);
            locale::set_timezone(&args.timezone);
        }
        Command::Networking(args) => {
            network::set_hostname(&args.hostname);
            network::enable_ipv6();
        }
        Command::Zram => {
            base::install_zram();
        }
        Command::Flatpak => {
            base::install_flatpak();
        }
        Command::Users { subcommand } => match subcommand {
            UsersSubcommand::NewUser(args) => {
                users::new_user(
                    &args.username,
                    &args.password,
                    false,
                );
            }
            UsersSubcommand::RootPass { password } => {
                users::root_pass(&password);
            }
        },
        Command::InstallParams(args) => {
            internal::install::install(args.cores, args.jobs, args.keep);
        }
        Command::Config { config } => {
            internal::config::read_config(config);
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
        _ => todo!()
    }
}
