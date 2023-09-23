use crate::args::TerminalSetup;
use crate::internal::*;

pub fn install_terminal_setup(terminal_setup: TerminalSetup) {
    log::debug!("Installing {:?}", terminal_setup);
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
        TerminalSetup::None => log::debug!("No terminal setup selected"),
    }
}

fn install_alacritty() {
    install(vec![
        "athena-alacritty-config",
    ]);
}

fn install_coolretroterm() {
    install(vec![
        "cool-retro-term",
    ]);
}

fn install_foot() {
    install(vec![
        "foot",
    ]);
}

fn install_gnometerminal() {
    install(vec![
        "gnome-terminal",
    ]);
}

fn install_kitty() {
    install(vec![
        "athena-kitty-config",
    ]);
}

fn install_konsole() {
    install(vec![
        "konsole",
    ]);
}

fn install_terminator() {
    install(vec![
        "terminator",
    ]);
}

fn install_terminology() {
    install(vec![
        "terminology",
    ]);
}

fn install_urxvt() {
    install(vec![
        "rxvt-unicode",
    ]);
}

fn install_xfce() {
    install(vec![
        "xfce4-terminal",
    ]);
}

fn install_xterm() {
    install(vec![
        "xterm",
    ]);
}
