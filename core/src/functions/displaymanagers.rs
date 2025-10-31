use crate::functions::desktops;
use crate::internal::services::enable_service;
use log::debug;
use shared::args::{DMSetup, is_arch, is_nix};
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_dm_setup(dm_setup: DMSetup) -> Vec<&'static str> {
    debug!("Selecting {dm_setup:?}");

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

    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  dmanager =.*",
                "  dmanager = \"gdm\";",
            ),
            "Set GDM",
        );        
    }

    packages
}

fn install_lightdm_neon() -> Vec<&'static str> {
    let packages = vec![
        "athena-lightdm-neon-theme",
    ];

    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  dmanager =.*",
                "  dmanager = \"lightdm\";",
            ),
            "Set LightDM",
        );        
    }

    packages
}

fn install_sddm() -> Vec<&'static str> {
    let packages = vec![
        "sddm-astronaut-theme",
    ];

    if is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  dmanager =.*",
                "  dmanager = \"sddm\";",
            ),
            "Set SDDM",
        ); 
    }

    packages
}

/**********************************/

pub fn configure_gdm(desktop: &str) {
    if ! desktop.contains("gnome") {
        files::rename_file("/mnt/usr/lib/udev/rules.d/61-gdm.rules", "/mnt/usr/lib/udev/rules.d/61-gdm.rules.bak");
        if is_arch() {
            desktops::disable_wsession("gnome-classic.desktop");
            desktops::disable_wsession("gnome-classic-wayland.desktop");
            desktops::disable_wsession("gnome-wayland.desktop");
        }
    }
    enable_service("gdm"); 
}

pub fn configure_lightdm_neon(desktop: &str) {
    lightdm_set_session(desktop);
    enable_service("lightdm");
}

pub fn configure_sddm() {
    if !is_nix() {
        // File creation and configuration can still happen here if needed
        files_eval(files::create_directory("/mnt/etc/sddm.conf.d"), "Create /mnt/etc/sddm.conf.d");
        files::create_file("/mnt/etc/sddm.conf");
        files::create_file("/mnt/etc/sddm.conf.d/virtualkbd.conf");
        files_eval(
            files::append_file(
                "/mnt/etc/sddm.conf",
                "[Theme]\nCurrent=sddm-astronaut-theme\n[XDisplay]\nDisplayCommand=/usr/share/sddm/scripts/Xsetup",
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
        files_eval(
            files::append_file(
                "/mnt/usr/share/sddm/scripts/Xsetup",
                "current=\"\"\nfor next in $(xrandr --listmonitors | grep -E \" *[0-9]+:.*\" | cut -d\" \" -f6); do\n  [ -z \"$current\" ] && current=$next && continue\n  xrandr --output \"$current\" --auto --output \"$next\" --auto --right-of \"$current\"\n  current=$next\ndone",
            ),
            "Add multimonitor support",
        );
        enable_service("sddm");
    }
}

pub fn configure_sddm_astronaut() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/astronaut.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"astronaut\";",
            ),
            "Set LightDM",
        );
    }
}

pub fn configure_sddm_blackhole() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/black_hole.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"black_hole\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_cyberpunk() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/cyberpunk.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"cyberpunk\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_cyborg() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/japanese_aesthetic.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"japanese_aesthetic\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_jake() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/jake_the_dog.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"jake_the_dog\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_kath() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/hyprland_kath.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"hyprland_kath\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_pixelsakura() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/pixel_sakura.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"pixel_sakura\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_postapocalypse() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/post-apocalyptic_hacker.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"post-apocalyptic_hacker\";",
            ),
            "Set LightDM",
        );        
    }
}

pub fn configure_sddm_purpleleaves() {
    if !is_nix() {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/sddm/themes/sddm-astronaut-theme/metadata.desktop",
                "ConfigFile=.*",
                "ConfigFile=Themes/purple_leaves.conf",
            ),
            "Set SDDM theme",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  sddmtheme =.*",
                "  sddmtheme = \"purple_leaves\";",
            ),
            "Set LightDM",
        );
    }
}

/**********************************/

fn lightdm_set_session(setdesktop: &str) {
    if setdesktop.contains("gnome") {
        files_eval(
            files::sed_file(
                "/mnt/etc/lightdm/lightdm.conf",
                "#user-session=.*",
                "user-session=gnome-wayland",
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