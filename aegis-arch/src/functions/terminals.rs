use crate::internal::install::install;
use shared::args::TerminalSetup;
use shared::args::PackageManager;
use shared::debug;

pub fn install_terminal_setup(terminal_setup: TerminalSetup) {
    debug!("Installing {:?}", terminal_setup);
    match terminal_setup {
        TerminalSetup::Alacritty => install_alacritty(),
        TerminalSetup::CoolRetroTerm => install_coolretroterm(),
        TerminalSetup::Foot => install_foot(),
        TerminalSetup::GnomeTerminal => install_gnometerminal(),
        TerminalSetup::Kitty => install_kitty(),
        TerminalSetup::Konsole => install_konsole(),
        TerminalSetup::Terminator => install_terminator(),
        TerminalSetup::Terminology => install_terminology(),
        TerminalSetup::Urxvt => install_urxvt(),
        TerminalSetup::Xfce => install_xfce(),
        TerminalSetup::Xterm => install_xterm(),
        TerminalSetup::None => debug!("No terminal setup selected"),
    }
}

fn install_alacritty() {
    install(PackageManager::Pacman, vec![
        "athena-alacritty-config",
    ]);
}

fn install_coolretroterm() {
    install(PackageManager::Pacman, vec![
        "cool-retro-term",
    ]);
}

fn install_foot() {
    install(PackageManager::Pacman, vec![
        "foot",
    ]);
}

fn install_gnometerminal() {
    install(PackageManager::Pacman, vec![
        "gnome-terminal",
    ]);
}

fn install_kitty() {
    install(PackageManager::Pacman, vec![
        "athena-kitty-config",
    ]);
}

fn install_konsole() {
    install(PackageManager::Pacman, vec![
        "konsole",
    ]);
}

fn install_terminator() {
    install(PackageManager::Pacman, vec![
        "terminator",
    ]);
}

fn install_terminology() {
    install(PackageManager::Pacman, vec![
        "terminology",
    ]);
}

fn install_urxvt() {
    install(PackageManager::Pacman, vec![
        "rxvt-unicode",
    ]);
}

fn install_xfce() {
    install(PackageManager::Pacman, vec![
        "xfce4-terminal",
    ]);
}

fn install_xterm() {
    install(PackageManager::Pacman, vec![
        "xterm",
    ]);
}
