use shared::args::BrowserSetup;
use shared::debug;

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
        "firefox-blackice",
    ]
}

fn install_brave() -> Vec<&'static str> {
    vec![
        "athena-brave-config",
    ]
}