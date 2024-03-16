use shared::args::TerminalSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_terminal_setup(terminal_setup: TerminalSetup) {
    debug!("Installing {:?}", terminal_setup);
    match terminal_setup {
        TerminalSetup::Alacritty => install_alacritty(),
        TerminalSetup::Kitty => install_kitty(),
        TerminalSetup::CoolRetroTerm => todo!(),
        TerminalSetup::Foot => todo!(),
        TerminalSetup::GnomeTerminal => todo!(),
        TerminalSetup::Konsole => todo!(),
        TerminalSetup::Terminator => todo!(),
        TerminalSetup::Terminology => todo!(),
        TerminalSetup::Urxvt => todo!(),
        TerminalSetup::Xfce => todo!(),
        TerminalSetup::Xterm => todo!(),
        TerminalSetup::None => debug!("No terminal setup selected"),
    }
}

fn install_alacritty() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  terminal =.*",
            "  terminal = \"alacritty\";",
        ),
        "Set Alacritty",
    );
}

fn install_kitty() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  terminal =.*",
            "  terminal = \"kitty\";",
        ),
        "Set Kitty",
    );
}
