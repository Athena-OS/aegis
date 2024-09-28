use shared::args::DMSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_dm_setup(dm_setup: DMSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", dm_setup);

    match dm_setup {
        DMSetup::Gdm => install_gdm(),
        DMSetup::LightDMNeon => install_lightdm_neon(),
        DMSetup::Sddm => install_sddm(),
        DMSetup::None => {
            debug!("No display manager setup selected");
            Vec::new() // Return empty vector if no DM setup is selected
        }
    }
}

fn install_gdm() -> Vec<&'static str> {
    let packages = vec![
        "athena-gdm-config",
    ];

    packages
}

fn install_lightdm_neon() -> Vec<&'static str> {
    let packages = vec![
        "athena-lightdm-neon-theme",
    ];

    packages
}

fn install_sddm() -> Vec<&'static str> {
    let packages = vec![
        "sddm-astronaut-theme",
    ];

    // File creation and configuration can still happen here if needed
    files::create_file("/mnt/etc/sddm.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/sddm.conf",
            "[Theme]\nCurrent=sddm-astronaut-theme",
        ),
        "Add astronaut theme",
    );

    packages
}