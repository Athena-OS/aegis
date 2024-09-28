use shared::args::ShellSetup;
use shared::debug;

pub fn install_shell_setup(shell_setup: ShellSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", shell_setup);

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
    vec![
        "bash", 
        "bash-completion", 
        "blesh-git",
    ]
}

fn install_fish() -> Vec<&'static str> {
    vec![
        "athena-fish",
    ]
}

fn install_zsh() -> Vec<&'static str> {
    vec![
        "athena-zsh",
    ]
}