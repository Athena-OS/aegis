use shared::args::ShellSetup;
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
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  mainShell =.*",
            "  mainShell = \"bash\";",
        ),
        "Set Bash",
    );
}

fn install_fish() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  mainShell =.*",
            "  mainShell = \"fish\";",
        ),
        "Set Fish",
    );
}

fn install_zsh() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  mainShell =.*",
            "  mainShell = \"zsh\";",
        ),
        "Set Zsh",
    );
}
