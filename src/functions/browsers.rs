use crate::args::BrowserSetup;
use crate::args::PackageManager;
use crate::internal::*;

pub fn install_browser_setup(browser_setup: BrowserSetup) {
    log::debug!("Installing {:?}", browser_setup);
    match browser_setup {
        BrowserSetup::Firefox => install_firefox(),
        BrowserSetup::Brave => install_brave(),
        BrowserSetup::Mullvad => install_mullvad(),
        BrowserSetup::None => log::debug!("No browser setup selected"),
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

fn install_mullvad() {
    install(PackageManager::Pacman, vec![
        "athena-mullvad-config",
    ]);
}