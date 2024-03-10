use crate::internal::install::install;
use crate::internal::services::enable_service;
use shared::args::DMSetup;
use shared::args::PackageManager;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_dm_setup(dm_setup: DMSetup) {
    debug!("Installing {:?}", dm_setup);
    match dm_setup {
        DMSetup::Gdm => install_gdm(),
        DMSetup::LightDMNeon => install_lightdm_neon(),
        DMSetup::Sddm => install_sddm(),
        DMSetup::None => debug!("No display manager setup selected"),
    }
}

fn install_gdm() {
    install(PackageManager::Pacman, vec![
        "athena-gdm-config",
    ]);
    enable_service("gdm");
}

fn install_lightdm_neon() {
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