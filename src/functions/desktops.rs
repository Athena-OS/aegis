use crate::args::DesktopSetup;
use crate::internal::exec::*;
use crate::internal::*;

pub fn install_desktop_setup(desktop_setup: DesktopSetup) {
    log::debug!("Installing {:?}", desktop_setup);
    match desktop_setup {
        DesktopSetup::Onyx => install_onyx(),
        DesktopSetup::Gnome => install_gnome(),
        DesktopSetup::Kde => install_kde(),
        DesktopSetup::Budgie => install_budgie(),
        DesktopSetup::Cinnamon => install_cinnamon(),
        DesktopSetup::Mate => install_mate(),
        DesktopSetup::Xfce => install_xfce(),
        DesktopSetup::Enlightenment => install_enlightenment(),
        DesktopSetup::Lxqt => install_lxqt(),
        DesktopSetup::Sway => install_sway(),
        DesktopSetup::I3 => install_i3(),
        DesktopSetup::Herbstluftwm => install_herbstluftwm(),
        DesktopSetup::Awesome => install_awesome(),
        DesktopSetup::Bspwm => install_bspwm(),
        DesktopSetup::Hyprland => install_hyprland(),
        DesktopSetup::None => log::debug!("No desktop setup selected"),
    }
    install_networkmanager();
}

fn install_networkmanager() {
    install(vec!["networkmanager"]);
    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("NetworkManager")],
        ),
        "Enable network manager",
    );
}

fn install_hyprland() {
    install(vec![
        "athena-hyprland-config",
    ]);
}

fn install_bspwm() {
    install(vec![
        "athena-bspwm-config",
    ]);
}

fn install_awesome() {
    install(vec![
        "xorg",
        "awesome",
        "dex",
        "rlwrap",
        "vicious",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-user-dirs",
    ]);
}

fn install_herbstluftwm() {
    install(vec![
        "xorg",
        "herbstluftwm",
        "dmenu",
        "dzen2",
        "xorg-xsetroot",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-user-dirs",
    ]);
}

fn install_i3() {
    install(vec![
        "xorg",
        "i3-wm",
        "dmenu",
        "i3lock",
        "i3status",
        "rxvt-unicode",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-user-dirs",
        "dex",
        "polkit-gnome",
    ]);
    files_eval(
        files::append_file("/mnt/etc/i3/config", "exec --no-startup-id dex -a\n"),
        "Add dex to i3 config for autostart",
    );
    files_eval(
        files::append_file(
            "/mnt/etc/i3/config",
            "exec --no-startup-id /usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1",
        ),
        "Add polkit gnome to i3 config",
    );
}

fn install_sway() {
    install(vec![
        "xorg-xwayland",
        "sway",
        "bemenu",
        "foot",
        "mako",
        "polkit",
        "swaybg",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "sddm",
        "xdg-user-dirs",
        "dex",
        "polkit-gnome",
    ]);
    files_eval(
        files::append_file("/mnt/etc/sway/config", "exec --no-startup-id dex -a\n"),
        "Add dex to sway config for autostart",
    );
    files_eval(
        files::append_file(
            "/mnt/etc/sway/config",
            "exec --no-startup-id /usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1",
        ),
        "Add polkit gnome to sway config",
    );
}

fn install_lxqt() {
    install(vec![
        "xorg",
        "lxqt",
        "breeze-icons",
        "nm-tray",
        "xscreensaver",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "sddm",
    ]);
}

fn install_enlightenment() {
    install(vec![
        "xorg",
        "enlightenment",
        "terminology",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
    ]);
}

fn install_xfce() {
    install(vec![
        "athena-xfce-config",
    ]);
}

fn install_mate() {
    install(vec![
        "athena-mate-config",
    ]);
}

fn install_cinnamon() {
    install(vec![
        "athena-cinnamon-config",
    ]);
}

fn install_budgie() {
    install(vec![
        "xorg",
        "budgie-desktop",
        "gnome",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-desktop-portal",
        "xdg-desktop-portal-gtk",
        "xdg-utils",
    ]);
}

fn install_kde() {
    install(vec![
        "athena-kde-config",
    ]);
}

fn install_gnome() {
    install(vec![
        "athena-gnome-config",
    ]);
}

fn install_onyx() {
    install(vec![
        "xorg",
        "onyx",
        "sushi",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "gdm",
    ]);
}