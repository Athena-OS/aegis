use shared::args::ThemeSetup;
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
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"akame\";",
        ),
        "Set Akame theme",
    );
}

fn install_cyborg() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"cyborg\";",
        ),
        "Set Cyborg theme",
    );
}

fn install_graphite() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"graphite\";",
        ),
        "Set Graphite theme",
    );
}

fn install_hackthebox() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"hackthebox\";",
        ),
        "Set Hack The Box theme",
    );
}

fn install_samurai() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"samurai\";",
        ),
        "Set Samurai theme",
    );
}

fn install_sweet() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"sweet\";",
        ),
        "Set Sweet theme",
    );
}

fn install_temple() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  theme =.*",
            "  theme = \"temple\";",
        ),
        "Set Temple theme",
    );
}