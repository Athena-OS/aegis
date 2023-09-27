use crate::args::ShellSetup;
use crate::args::PackageManager;
use crate::internal::*;

pub fn install_shell_setup(shell_setup: ShellSetup) {
    log::debug!("Installing {:?}", shell_setup);
    match shell_setup {
        ShellSetup::Bash => install_bash(),
        ShellSetup::Fish => install_fish(),
        ShellSetup::Zsh => install_zsh(),
        ShellSetup::None => log::debug!("No shell setup selected"),
    }
}

fn install_bash() {
    install(PackageManager::Pacman, vec![
        "bash", "bash-completion", "blesh-git",
    ]);
}

fn install_fish() {
    install(PackageManager::Pacman, vec![
        "athena-fish",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.local/share/applications/shell.desktop",
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

fn install_zsh() {
    install(PackageManager::Pacman, vec![
        "athena-zsh",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.local/share/applications/shell.desktop",
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