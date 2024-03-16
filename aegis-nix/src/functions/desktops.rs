use shared::args::DesktopSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_desktop_setup(desktop_setup: DesktopSetup) {
    debug!("Installing {:?}", desktop_setup);
    match desktop_setup {
        DesktopSetup::Gnome => install_gnome(),
        DesktopSetup::Cinnamon => install_cinnamon(),
        DesktopSetup::Mate => install_mate(),
        DesktopSetup::XfceRefined => install_xfce_refined(),
        DesktopSetup::XfcePicom => install_xfce_picom(),
        DesktopSetup::Onyx => todo!(),
        DesktopSetup::Kde => todo!(),
        DesktopSetup::Budgie => todo!(),
        DesktopSetup::Enlightenment => todo!(),
        DesktopSetup::Lxqt => todo!(),
        DesktopSetup::Sway => todo!(),
        DesktopSetup::I3 => todo!(),
        DesktopSetup::Herbstluftwm => todo!(),
        DesktopSetup::Awesome => todo!(),
        DesktopSetup::Bspwm => todo!(),
        DesktopSetup::Hyprland => todo!(),
        DesktopSetup::None => todo!(),
    }
}

fn install_xfce_refined() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  desktop =.*",
            "  desktop = \"xfce\";",
        ),
        "Set XFCE",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/home-manager/desktops/xfce/default.nix",
            "athena.desktops.xfce.refined =.*",
            "athena.desktops.xfce.refined = true;",
        ),
        "Set XFCE Refined",
    );
}

fn install_xfce_picom() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  desktop =.*",
            "  desktop = \"xfce\";",
        ),
        "Set XFCE",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/home-manager/desktops/xfce/default.nix",
            "athena.desktops.xfce.refined =.*",
            "athena.desktops.xfce.refined = false;",
        ),
        "Set XFCE Picom",
    );
}

fn install_gnome() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  desktop =.*",
            "  desktop = \"gnome\";",
        ),
        "Set GNOME",
    );
}

fn install_cinnamon() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  desktop =.*",
            "  desktop = \"cinnamon\";",
        ),
        "Set Cinnamon",
    );
}

fn install_mate() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  desktop =.*",
            "  desktop = \"mate\";",
        ),
        "Set MATE",
    );
}
