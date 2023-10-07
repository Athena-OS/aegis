use crate::args::DMSetup;
use crate::args::PackageManager;
use crate::internal::{files, files_eval, install};
use crate::internal::services::enable_service;

pub fn install_dm_setup(dm_setup: DMSetup) {
    log::debug!("Installing {:?}", dm_setup);
    match dm_setup {
        DMSetup::Gdm => install_gdm(),
        DMSetup::LightDM => install_lightdm(),
        DMSetup::Sddm => install_sddm(),
        DMSetup::None => log::debug!("No display manager setup selected"),
    }
}

fn install_gdm() {
    install(PackageManager::Pacman, vec![
        "athena-gdm-config",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/gdm/custom.conf",
            ".*WaylandEnable=.*",
            "WaylandEnable=false",
        ),
        "Apply GDM",
    );
    enable_service("gdm");
}

fn install_lightdm() {
    install(PackageManager::Pacman, vec![
        "athena-lightdm-neon-theme",
    ]);
    enable_service("lightdm");
}

fn install_sddm() {
    install(PackageManager::Pacman, vec![
        "sddm-theme-astronaut",
    ]);
    files::create_file("/mnt/etc/sddm.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/sddm.conf",
            "[Theme]\nCurrent=astronaut",
        ),
        "Add astronaut theme",
    );
    enable_service("sddm");
}