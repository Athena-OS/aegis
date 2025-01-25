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
    files::create_file("/mnt/etc/sddm.conf.d/virtualkbd.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/sddm.conf",
            "[Theme]\nCurrent=sddm-astronaut-theme",
        ),
        "Add astronaut theme",
    );
    files_eval(
        files::append_file(
            "/mnt/etc/sddm.conf.d/virtualkbd.conf",
            "[General]\nInputMethod=qtvirtualkeyboard",
        ),
        "Add virtual keyboard support",
    );
    enable_service("sddm");
}

pub fn configure_sddm_astronaut() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/astronaut.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_blackhole() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/black_hole.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_cyberpunk() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/cyberpunk.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_cyborg() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/japanese_aesthetic.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_jake() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/jake_the_dog.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_kath() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/hyprland_kath.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_pixelsakura() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/pixel_sakura.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_postapocalypse() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/post-apocalyptic_hacker.conf",
        ),
        "Set SDDM theme",
    );
}

pub fn configure_sddm_purpleleaves() {
    files_eval(
        files::sed_file(
            "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
            "ConfigFile=.*",
            "ConfigFile=Themes/purple_leaves.conf",
        ),
        "Set SDDM theme",
    );
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