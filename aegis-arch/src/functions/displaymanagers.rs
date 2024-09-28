use crate::desktops;
use crate::internal::services::enable_service;
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

    packages
}

/**********************************/

pub fn configure_gdm(desktop: &str) {
    if ! desktop.contains("gnome") {
        files::rename_file("/mnt/usr/lib/udev/rules.d/61-gdm.rules", "/mnt/usr/lib/udev/rules.d/61-gdm.rules.bak");
        desktops::disable_xsession("gnome.desktop");
        desktops::disable_xsession("gnome-xorg.desktop");
        desktops::disable_wsession("gnome.desktop");
        desktops::disable_wsession("gnome-wayland.desktop");
        // Note that gnome-classic sessions belong to gnome-shell-extensions pkg that is not installed by GDM
    }
    else {
        files_eval(
            files::sed_file(
                "/mnt/etc/gdm/custom.conf",
                ".*WaylandEnable=.*",
                "WaylandEnable=false",
            ),
            "Disable Wayland in GNOME",
        );
    }
    enable_service("gdm"); 
}

pub fn configure_lightdm_neon(desktop: &str) {
    lightdm_set_session(desktop);
    enable_service("lightdm");
}

pub fn configure_sddm() {
    // File creation and configuration can still happen here if needed
    files::create_file("/mnt/etc/sddm.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/sddm.conf",
            "[Theme]\nCurrent=sddm-astronaut-theme",
        ),
        "Add astronaut theme",
    );
    enable_service("sddm");
}

/**********************************/

fn lightdm_set_session(setdesktop: &str) {
    if setdesktop.contains("gnome") {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=gnome-xorg",
            ),
            "Apply GNOME User Session on LightDM",
        );
    }
    if setdesktop.contains("xfce") {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=xfce",
            ),
            "Apply XFCE User Session on LightDM",
        );
    }
    if setdesktop == "hyprland" {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=Hyprland",
            ),
            "Apply Hyprland User Session on LightDM",
        );
    }
}