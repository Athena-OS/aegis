use shared::args::DesktopSetup;
use shared::debug;
use shared::files;

pub fn install_desktop_setup(desktop_setup: DesktopSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", desktop_setup);

    match desktop_setup {
        DesktopSetup::Onyx => install_onyx(),
        DesktopSetup::Gnome => install_gnome(),
        DesktopSetup::Kde => install_kde(),
        DesktopSetup::Budgie => install_budgie(),
        DesktopSetup::Cinnamon => install_cinnamon(),
        DesktopSetup::Mate => install_mate(),
        DesktopSetup::XfceRefined => install_xfce_refined(),
        DesktopSetup::XfcePicom => install_xfce_picom(),
        DesktopSetup::Enlightenment => install_enlightenment(),
        DesktopSetup::Lxqt => install_lxqt(),
        DesktopSetup::Sway => install_sway(),
        DesktopSetup::I3 => install_i3(),
        DesktopSetup::Herbstluftwm => install_herbstluftwm(),
        DesktopSetup::Awesome => install_awesome(),
        DesktopSetup::Bspwm => install_bspwm(),
        DesktopSetup::Hyprland => install_hyprland(),
        DesktopSetup::None => {
            debug!("No desktop setup selected");
            Vec::new() // Return empty vector for "None"
        }
    }
}

fn install_hyprland() -> Vec<&'static str> {
    vec![
        "athena-hyprland-config",
    ]
}

fn install_bspwm() -> Vec<&'static str> {
    vec![
        "athena-bspwm-config",
    ]
}

fn install_awesome() -> Vec<&'static str> {
    vec![
        "xorg",
        "awesome",
        "dex",
        "rlwrap",
        "vicious",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-user-dirs",
    ]
}

fn install_herbstluftwm() -> Vec<&'static str> {
    vec![
        "xorg",
        "herbstluftwm",
        "dmenu",
        "dzen2",
        "xorg-xsetroot",
        "lightdm",
        "lightdm-gtk-greeter",
        "lightdm-gtk-greeter-settings",
        "xdg-user-dirs",
    ]
}

fn install_i3() -> Vec<&'static str> {
    vec![
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
    ]
}

fn install_sway() -> Vec<&'static str> {
    vec![
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
    ]
}

fn install_lxqt() -> Vec<&'static str> {
    vec![
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
    ]
}

fn install_enlightenment() -> Vec<&'static str> {
    vec![
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
    ]
}

fn install_xfce_refined() -> Vec<&'static str> {
    vec![
        "athena-xfce-refined",
    ]
}

fn install_xfce_picom() -> Vec<&'static str> {
    vec![
        "athena-xfce-picom",
    ]
}

fn install_mate() -> Vec<&'static str> {
    vec![
        "athena-mate-base",
    ]
}

fn install_cinnamon() -> Vec<&'static str> {
    vec![
        "athena-cinnamon-base",
    ]
}

fn install_budgie() -> Vec<&'static str> {
    vec![
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
    ]
}

fn install_kde() -> Vec<&'static str> {
    vec![
        "athena-kde-base",
    ]
}

fn install_gnome() -> Vec<&'static str> {
    vec![
        "athena-gnome-config",
    ]
}

fn install_onyx() -> Vec<&'static str> {
    vec![
        "xorg",
        "onyx",
        "sushi",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "wireplumber",
        "gdm",
    ]
}

/**********************************/

pub fn configure_gnome() {
    disable_xsession("gnome.desktop");
    disable_xsession("gnome-classic.desktop");
    disable_xsession("gnome-classic-xorg.desktop");
    disable_wsession("gnome.desktop");
    disable_wsession("gnome-wayland.desktop");
    disable_wsession("gnome-classic.desktop");
    disable_wsession("gnome-classic-wayland.desktop");
}

/**********************************/

pub fn disable_xsession(session: &str) {
    debug!("Disabling {}", session);
    files::rename_file(&("/mnt/usr/share/xsessions/".to_owned()+session), &("/mnt/usr/share/xsessions/".to_owned()+session+".disable"));
}

pub fn disable_wsession(session: &str) {
    debug!("Disabling {}", session);
    files::rename_file(&("/mnt/usr/share/wayland-sessions/".to_owned()+session), &("/mnt/usr/share/wayland-sessions/".to_owned()+session+".disable"));
}