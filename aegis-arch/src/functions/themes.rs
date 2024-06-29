use crate::internal::install::install;
use shared::args::ThemeSetup;
use shared::args::PackageManager;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_theme_setup(theme_setup: ThemeSetup) {
    debug!("Installing {:?}", theme_setup);
    match theme_setup {
        ThemeSetup::Akame => install_akame(),
        ThemeSetup::Cyborg => install_cyborg(),
        ThemeSetup::Graphite => install_graphite(),
        ThemeSetup::HackTheBox => install_hackthebox(),
        ThemeSetup::Samurai => install_samurai(),
        ThemeSetup::Sweet => install_sweet(),
        ThemeSetup::Temple => install_temple(),
        ThemeSetup::None => debug!("No theme setup selected"),
    }
}

fn install_akame() {
    install(PackageManager::Pacman, vec![
        "athena-akame-theme",
    ]);
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

fn install_cyborg() {
    install(PackageManager::Pacman, vec![
        "athena-cyborg-theme",
    ]);
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

fn install_graphite() {
    install(PackageManager::Pacman, vec![
        "athena-graphite-theme",
    ]);
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

fn install_hackthebox() {
    install(PackageManager::Pacman, vec![
        "athena-htb-theme",
    ]);
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

fn install_samurai() {
    install(PackageManager::Pacman, vec![
        "athena-samurai-theme",
    ]);
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

fn install_sweet() {
    install(PackageManager::Pacman, vec![
        "athena-sweetdark-theme",
    ]);
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

fn install_temple() {
    install(PackageManager::Pacman, vec![
        "athena-temple-theme",
    ]);
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