use log::debug;
use shared::args::{is_arch, is_nix, ShellSetup};
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_shell_setup(shell_setup: ShellSetup) -> Vec<&'static str> {
    debug!("Selecting {shell_setup:?}");

    match shell_setup {
        ShellSetup::Bash => install_bash(),
        ShellSetup::Fish => install_fish(),
        ShellSetup::Zsh => install_zsh(),
        ShellSetup::None => {
            debug!("No shell setup selected");
            Vec::new() // Return empty vector if no shell setup is selected
        }
    }
}

fn install_bash() -> Vec<&'static str> {
    let mut pkgs = vec!["bash", "bash-completion"];
    if is_arch() {
        pkgs.push("blesh-git");
    }
    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  mainShell =.*",
                "  mainShell = \"bash\";",
            ),
            "Set Bash",
        );        
    }
    pkgs
}

fn install_fish() -> Vec<&'static str> {
    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  mainShell =.*",
                "  mainShell = \"fish\";",
            ),
            "Set Fish",
        );
    }
    vec![
        "athena-fish",
    ]
}

fn install_zsh() -> Vec<&'static str> {
    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  mainShell =.*",
                "  mainShell = \"zsh\";",
            ),
            "Set Zsh",
        );
    }
    vec![
        "athena-zsh",
    ]
}

/**********************************/

pub fn configure_fish() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/applications/shell.desktop",
            "Bash",
            "Fish",
        ),
        "Apply FISH shell on .desktop shell file",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.bashrc",
            "export SHELL=.*",
            r"export SHELL=$(which fish)",
        ),
        "Apply FISH shell",
    );
}

pub fn configure_zsh() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/applications/shell.desktop",
            "Bash",
            "Zsh",
        ),
        "Apply ZSH shell on .desktop shell file",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.bashrc",
            "export SHELL=.*",
            r"export SHELL=$(which zsh)",
        ),
        "Apply ZSH shell",
    );
}