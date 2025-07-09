mod functions;
mod internal;
use crate::functions::*;
use shared::args::{BootloaderSubcommand, Command, Cli, UsersSubcommand};
use shared::clap::Parser;
use shared::exec::check_if_root;
use shared::human_panic;
use shared::log::info;
use shared::logging;
use shared::partition;

fn main() -> Result<(), i32> {
    check_if_root();
    human_panic::setup_panic!();
    let cli = Cli::parse();
    info!("verbose: {}", cli.verbose);
    let log_file_path = "/tmp/aegis";
    logging::init(cli.verbose, log_file_path);
    // menu choice
    match cli.command {
        Command::Partition(args) => {
            let mut partitions = args.partitions;
            partition::partition(
                args.device,
                args.mode,
                args.encrypt_check,
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
        }
        Command::Config { config } => {
            let exit_code = internal::config::read_config(config);
            if exit_code != 0 {
                return Err(exit_code);
            }
        }
        Command::Desktops { desktop } => {
            desktops::install_desktop_setup(desktop);
        }
        Command::Themes { design } => {
            themes::install_theme_setup(design);
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
        },
        _ => todo!() //Do nothing for all those Command:: specified in shared/args.rs but not specifically implemented in athena-nix (because useless)
    }
    Ok(())
}
