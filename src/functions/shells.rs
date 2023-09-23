use crate::args::ShellSetup;
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
    install(vec![
        "bash", "bash-completion", "blesh-git"
    ]);
}

fn install_fish() {
    install(vec![
        "athena-fish",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.local/share/applications/*",
            r"Bash",
            r"Fish",
        ),
        "Apply FISH shell on .desktop user files",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.bashrc",
            r"export SHELL=.*",
            r"export SHELL=\$(which fish)",
        ),
        "Apply FISH shell",
    );
    files::create_file("/mnt/etc/profile.d/shell.sh");
    files_eval(
        files::append_file("/mnt/etc/profile.d/shell.sh", r"export SHELL=\$(which fish)"),
        "Add SHELL variable on profile.d script",
    );
}

fn install_zsh() {
    install(vec![
        "athena-zsh",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.local/share/applications/*",
            r"Bash",
            r"Zsh",
        ),
        "Apply ZSH shell on .desktop user files",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.bashrc",
            r"export SHELL=.*",
            r"export SHELL=\$(which zsh)",
        ),
        "Apply ZSH shell",
    );
    files::create_file("/mnt/etc/profile.d/shell.sh");
    files_eval(
        files::append_file("/mnt/etc/profile.d/shell.sh", r"export SHELL=\$(which zsh)"),
        "Add SHELL variable on profile.d script",
    );
}