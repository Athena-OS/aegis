use crate::internal::install::install;
use shared::args::BrowserSetup;
use shared::args::PackageManager;
use shared::debug;

pub fn install_browser_setup(browser_setup: BrowserSetup) {
    debug!("Installing {:?}", browser_setup);
    match browser_setup {
        BrowserSetup::Firefox => install_firefox(),
        BrowserSetup::Brave => install_brave(),
        BrowserSetup::None => debug!("No browser setup selected"),
    }
}

fn install_firefox() {
    install(PackageManager::Pacman, vec![
        "athena-firefox-config",
    ]);
}

fn install_brave() {
    install(PackageManager::Pacman, vec![
        "athena-brave-config",
    ]);
}