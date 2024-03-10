use crate::internal::install::install;
use shared::args::DesktopSetup;
use shared::args::PackageManager;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_desktop_setup(desktop_setup: DesktopSetup) {
    debug!("Installing {:?}", desktop_setup);
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
        DesktopSetup::None => debug!("No desktop setup selected"),
    }
}

fn install_hyprland() {
    install(PackageManager::Pacman, vec![
        "athena-hyprland-config",
    ]);
}

fn install_bspwm() {
    install(PackageManager::Pacman, vec![
        "athena-bspwm-config",
    ]);
}

fn install_awesome() {
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
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

fn install_xfce_refined() {
    install(PackageManager::Pacman, vec![
        "athena-xfce-config",
    ]);
}

fn install_xfce_picom() {
    install(PackageManager::Pacman, vec![
        "athena-xfce-picom",
    ]);
}

fn install_mate() {
    install(PackageManager::Pacman, vec![
        "athena-mate-base",
    ]);
}

fn install_cinnamon() {
    install(PackageManager::Pacman, vec![
        "athena-cinnamon-base",
    ]);
}

fn install_budgie() {
    install(PackageManager::Pacman, vec![
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
    install(PackageManager::Pacman, vec![
        "athena-kde-base",
    ]);
}

fn install_gnome() {
    install(PackageManager::Pacman, vec![
        "athena-gnome-config",
    ]);
}

fn install_onyx() {
    install(PackageManager::Pacman, vec![
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