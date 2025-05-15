use shared::args::ThemeSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_theme_setup(theme_setup: ThemeSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", theme_setup);

    match theme_setup {
        ThemeSetup::Cyborg => install_cyborg(),
        ThemeSetup::Graphite => install_graphite(),
        ThemeSetup::HackTheBox => install_hackthebox(),
        ThemeSetup::RedMoon => install_redmoon(),
        ThemeSetup::Samurai => install_samurai(),
        ThemeSetup::Sweet => install_sweet(),
        ThemeSetup::Temple => install_temple(),
        ThemeSetup::None => {
            debug!("No theme setup selected");
            Vec::new() // Return empty vector if no theme setup is selected
        }
    }
}

fn install_cyborg() -> Vec<&'static str> {
    vec![
        "athena-cyborg-theme",
    ]
}

fn install_graphite() -> Vec<&'static str> {
    vec![
        "athena-graphite-theme",
    ]
}

fn install_hackthebox() -> Vec<&'static str> {
    vec![
        "athena-htb-theme",
    ]
}

fn install_redmoon() -> Vec<&'static str> {
    vec![
        "athena-redmoon-theme",
    ]
}

fn install_samurai() -> Vec<&'static str> {
    vec![
        "athena-samurai-theme",
    ]
}

fn install_sweet() -> Vec<&'static str> {
    vec![
        "athena-sweetdark-theme",
    ]
}

fn install_temple() -> Vec<&'static str> {
    vec![
        "athena-temple-theme",
    ]
}

/**********************************/

pub fn configure_cyborg() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Gruvbox Material Dark\",",
        ),
        "Apply Gruvbox Material Dark VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'gold'",
        ),
        "Apply Gold Tmux theme",
    );
}

pub fn configure_graphite() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Just Black\",",
        ),
        "Apply Just Black VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'snow'",
        ),
        "Apply Snow Tmux theme",
    );
}

pub fn configure_hackthebox() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Hack The Box\",",
        ),
        "Apply Hack The Box VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'forest'",
        ),
        "Apply Forest Tmux theme",
    );
}

pub fn configure_redmoon() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"red-blood\",",
        ),
        "Apply Red Blood VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'redwine'",
        ),
        "Apply Redwine Tmux theme",
    );
}

pub fn configure_samurai() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Tokyo Night Storm\",",
        ),
        "Apply Tokyo Night Storm VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'sky'",
        ),
        "Apply Sky Tmux theme",
    );
}

pub fn configure_sweet() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Radical\",",
        ),
        "Apply Radical VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'violet'",
        ),
        "Apply Violet Tmux theme",
    );
}

pub fn configure_temple() {
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Tokyo Night Storm\",",
        ),
        "Apply Tokyo Night Storm VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'sky'",
        ),
        "Apply Sky Tmux theme",
    );
}