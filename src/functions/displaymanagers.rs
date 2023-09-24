use crate::args::DMSetup;
use crate::internal::exec::*;
use crate::internal::*;

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
    install(vec![
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
    enable_dm("gdm");
}

fn install_lightdm() {
    install(vec![
        "athena-lightdm-webkit-theme-aether",
    ]);
    files_eval(
        files::sed_file(
            "/mnt/etc/lightdm/lightdm-webkit2-greeter.conf",
            "^webkit_theme .*",
            r"c\webkit_theme = lightdm-webkit-theme-aether",
        ),
        "Apply LightDM",
    );
    enable_dm("lightdm");
}

fn install_sddm() {
    install(vec![
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
    enable_dm("sddm");
}

fn enable_dm(dm: &str) {
    log::debug!("Enabling {}", dm);
    exec_eval(
        exec_chroot("systemctl", vec![String::from("enable"), String::from(dm)]),
        format!("Enable {}", dm).as_str(),
    );
}