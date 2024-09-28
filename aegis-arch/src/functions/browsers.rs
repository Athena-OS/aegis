use shared::args::BrowserSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_browser_setup(browser_setup: BrowserSetup) -> Vec<&'static str> {
    debug!("Selecting {:?}", browser_setup);

    match browser_setup {
        BrowserSetup::Firefox => install_firefox(),
        BrowserSetup::Brave => install_brave(),
        BrowserSetup::None => {
            debug!("No browser setup selected");
            Vec::new()  // Return an empty vector if no browser setup is selected
        }
    }
}

fn install_firefox() -> Vec<&'static str> {
    vec![
        "athena-firefox-config",
    ]
}

fn install_brave() -> Vec<&'static str> {
    vec![
        "athena-brave-config",
    ]
}

/**********************************/

pub fn configure_firefox(desktop: &str) {
    if desktop.contains("gnome") {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                "\\{\\\\\"name\\\\\":\\\\\"Brave\\\\\",\\\\\"icon\\\\\":\\\\\"/usr/share/icons/hicolor/scalable/apps/brave.svg\\\\\",\\\\\"type\\\\\":\\\\\"Command\\\\\",\\\\\"data\\\\\":\\{\\\\\"command\\\\\":\\\\\"brave\\\\\"\\},\\\\\"angle\\\\\":-1\\}",
                "{\\\"name\\\":\\\"Firefox ESR\\\",\\\"icon\\\":\\\"/usr/share/icons/hicolor/scalable/apps/firefox-logo.svg\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{\\\"command\\\":\\\"firefox-esr\\\"},\\\"angle\\\":-1}",
            ),
            "Apply Browser info on dconf shell",
        );
    }
}

pub fn configure_brave(desktop: &str) {
    if desktop.contains("gnome") {
        files_eval(
            files::sed_file(
                "/mnt/usr/share/athena-gnome-config/dconf-shell.ini",
                "\\{\\\\\"name\\\\\":\\\\\"Firefox ESR\\\\\",\\\\\"icon\\\\\":\\\\\"/usr/share/icons/hicolor/scalable/apps/firefox-logo.svg\\\\\",\\\\\"type\\\\\":\\\\\"Command\\\\\",\\\\\"data\\\\\":\\{\\\\\"command\\\\\":\\\\\"firefox-esr\\\\\"\\},\\\\\"angle\\\\\":-1\\}",
                "{\\\"name\\\":\\\"Brave\\\",\\\"icon\\\":\\\"/usr/share/icons/hicolor/scalable/apps/brave.svg\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{\\\"command\\\":\\\"brave\\\"},\\\"angle\\\":-1}",
            ),
            "Apply Browser info on dconf shell",
        );
    }
}