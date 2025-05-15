use shared::args::TerminalSetup;
use shared::debug;
use shared::exec::exec;
use shared::files::sed_file;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;

pub fn install_terminal_setup(terminal_setup: TerminalSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", terminal_setup);

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
        TerminalSetup::None => {
            debug!("No terminal setup selected");
            Vec::new() // Return empty vector if no terminal setup is selected
        }
    }
}

fn install_alacritty() -> Vec<&'static str> {
    vec![
        "athena-alacritty-config",
    ]
}

fn install_coolretroterm() -> Vec<&'static str> {
    vec![
        "cool-retro-term",
    ]
}

fn install_foot() -> Vec<&'static str> {
    vec![
        "foot",
    ]
}

fn install_gnometerminal() -> Vec<&'static str> {
    vec![
        "gnome-terminal",
    ]
}

fn install_kitty() -> Vec<&'static str> {
    vec![
        "athena-kitty-config",
    ]
}

fn install_konsole() -> Vec<&'static str> {
    vec![
        "konsole",
    ]
}

fn install_terminator() -> Vec<&'static str> {
    vec![
        "terminator",
    ]
}

fn install_terminology() -> Vec<&'static str> {
    vec![
        "terminology",
    ]
}

fn install_urxvt() -> Vec<&'static str> {
    vec![
        "rxvt-unicode",
    ]
}

fn install_xfce() -> Vec<&'static str> {
    vec![
        "xfce4-terminal",
    ]
}

fn install_xterm() -> Vec<&'static str> {
    vec![
        "xterm",
    ]
}

/**********************************/

pub fn configure_terminal(term: String, desktop: &str) {
    exec_eval(
        exec( // Using exec instead of exec_chroot because in exec_chroot, these sed arguments need some chars to be escaped
            "sed",
            vec![
                String::from("-i"),
                String::from("-e"),
                format!("s/^TERMINAL_EXEC=.*/TERMINAL_EXEC=\"{}\"/g", &(term.clone()+" "+if term == "gnome-terminal" { "--" } else { "-e" })),
                String::from("/mnt/usr/bin/shell-rocket"),
            ],
        ),
        "Set terminal on shell rocket",
    );
    files_eval(
        sed_file(
            "/mnt/usr/share/applications/shell.desktop",
            "alacritty",
            &term,
        ),
        "Set terminal call on shell.desktop file",
    );
    if desktop.contains("gnome") {
        files_eval(
            sed_file(
                "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                "alacritty",
                &term,
            ),
            "Set terminal call on dconf file",
        );
    }
}