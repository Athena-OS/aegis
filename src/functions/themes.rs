use crate::args::ThemeSetup;
use crate::internal::*;

pub fn install_theme_setup(theme_setup: ThemeSetup) {
    log::debug!("Installing {:?}", theme_setup);
    match theme_setup {
        ThemeSetup::Akame => install_akame(),
        ThemeSetup::Samurai => install_samurai(),
        ThemeSetup::Graphite => install_graphite(),
        ThemeSetup::Cyborg => install_cyborg(),
        ThemeSetup::Sweet => install_sweet(),
        ThemeSetup::XXE => install_xxe(),
        ThemeSetup::HackTheBox => install_htb(),
        ThemeSetup::None => log::debug!("No theme setup selected"),
    }
}

fn install_akame() {
    install(vec![
        "athena-akame-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"red-blood\",",
        ),
        "Apply Red Blood VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'redwine'",
        ),
        "Apply Redwine Tmux theme",
    );
}

fn install_samurai() {
    install(vec![
        "athena-blue-eyes-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Tokyo Night Storm\",",
        ),
        "Apply Tokyo Night Storm VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'sky'",
        ),
        "Apply Sky Tmux theme",
    );
}

fn install_graphite() {
    install(vec![
        "athena-graphite-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Just Black\",",
        ),
        "Apply Just Black VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'snow'",
        ),
        "Apply Snow Tmux theme",
    );
}

fn install_cyborg() {
    install(vec![
        "athena-gruvbox-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Gruvbox Material Dark\",",
        ),
        "Apply Gruvbox Material Dark VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'gold'",
        ),
        "Apply Gold Tmux theme",
    );
}

fn install_sweet() {
    install(vec![
        "athena-sweet-dark-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Radical\",",
        ),
        "Apply Radical VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'violet'",
        ),
        "Apply Violet Tmux theme",
    );
}

fn install_xxe() {
    install(vec![
        "athena-xxe-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Gruvbox Material Dark\",",
        ),
        "Apply Gruvbox Material Dark VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'gold'",
        ),
        "Apply Gold Tmux theme",
    );
}

fn install_htb() {
    install(vec![
        "athena-htb-theme",
    ]);
    files_eval(
        files::sed_file(
            "/etc/skel/.config/VSCodium/User/settings.json",
            "\"workbench.colorTheme\":.*",
            "\"workbench.colorTheme\": \"Hack The Box\",",
        ),
        "Apply Hack The Box VSCodium theme",
    );
    files_eval(
        files::sed_file(
            "/etc/skel/.tmux.conf",
            "set -g @tmux_power_theme.*",
            "set -g @tmux_power_theme 'forest'",
        ),
        "Apply Forest Tmux theme",
    );
}