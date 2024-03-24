mod functions;
mod internal;
//use crate::internal::secure;
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
    match cli.command {
        Command::Partition(args) => {
            let mut partitions = args.partitions;
            partition::partition(
                args.device,
                args.mode,
                args.encrypt_auto,
                args.efi,
                args.swap,
                args.swap_size,
                &mut partitions,
            );
        }
        Command::InstallBase => {
            base::install_base_packages();
        }
        Command::InstallPackages(args) => {
            base::install_packages(args.kernel);
        }
        Command::GenFstab => {
            base::genfstab();
        }
        //Command::SetupSnapper => base::setup_snapper(),
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
        /*Command::Hardened => {
            secure::secure_password_config();
            secure::secure_ssh_config();
        }*/
        Command::Users { subcommand } => match subcommand {
            UsersSubcommand::NewUser(args) => {
                users::new_user(
                    &args.username,
                    args.hasroot,
                    &args.password,
                    false,
                    &args.shell,
                );
            }
            UsersSubcommand::RootPass { password } => {
                users::root_pass(&password);
            }
        },
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
        Command::InstallParams(args) => {
            //internal::install::install(args.cores, args.jobs);
            println!("{} {}", args.cores, args.jobs); //Just to delete the warning about unused args variable
            todo!()
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
        Command::EnableServices => {
            base::enable_system_services();
        }
    }
}
