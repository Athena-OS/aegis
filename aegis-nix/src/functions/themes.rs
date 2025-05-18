use shared::args::ThemeSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_theme_setup(theme_setup: ThemeSetup) {
    debug!("Installing {:?}", theme_setup);
    match theme_setup {
        ThemeSetup::Cyborg => install_cyborg(),
        ThemeSetup::Graphite => install_graphite(),
        ThemeSetup::HackTheBox => install_hackthebox(),
        ThemeSetup::RedMoon => install_redmoon(),
        ThemeSetup::Samurai => install_samurai(),
        ThemeSetup::Sweet => install_sweet(),
        ThemeSetup::Temple => install_temple(),
        ThemeSetup::None => debug!("No design setup selected"),
    }
}

fn install_cyborg() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"cyborg\";",
        ),
        "Set Cyborg design",
    );
}

fn install_graphite() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"graphite\";",
        ),
        "Set Graphite design",
    );
}

fn install_hackthebox() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"hackthebox\";",
        ),
        "Set Hack The Box design",
    );
}

fn install_redmoon() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"redmoon\";",
        ),
        "Set Red Moon design",
    );
}

fn install_samurai() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"samurai\";",
        ),
        "Set Samurai design",
    );
}

fn install_sweet() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"sweet\";",
        ),
        "Set Sweet design",
    );
}

fn install_temple() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"temple\";",
        ),
        "Set Temple design",
    );
}