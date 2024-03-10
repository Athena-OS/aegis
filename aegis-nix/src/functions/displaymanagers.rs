use shared::args::DMSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_dm_setup(dm_setup: DMSetup) {
    debug!("Installing {:?}", dm_setup);
    match dm_setup {
        DMSetup::Gdm => install_gdm(),
        DMSetup::LightDMNeon => install_lightdm_neon(),
        DMSetup::Sddm => todo!(),
        DMSetup::None => debug!("No display manager setup selected"),
    }
}

fn install_gdm() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "dmanager =.*",
            "dmanager = \"gdm\";",
        ),
        "Set GDM",
    );
}

fn install_lightdm_neon() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "dmanager =.*",
            "dmanager = \"lightdm\";",
        ),
        "Set LightDM",
    );
}