use crate::internal::install::install;
use shared::args::ShellSetup;
use shared::args::PackageManager;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_shell_setup(shell_setup: ShellSetup) {
    debug!("Installing {:?}", shell_setup);
    match shell_setup {
        ShellSetup::Bash => install_bash(),
        ShellSetup::Fish => install_fish(),
        ShellSetup::Zsh => install_zsh(),
        ShellSetup::None => debug!("No shell setup selected"),
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

fn install_zsh() {
    install(PackageManager::Pacman, vec![
        "athena-zsh",
    ]);
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