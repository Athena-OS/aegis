use shared::args::BrowserSetup;
use shared::debug;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn install_browser_setup(browser_setup: BrowserSetup) {
    debug!("Installing {:?}", browser_setup);
    match browser_setup {
        BrowserSetup::Firefox => install_firefox(),
        BrowserSetup::Brave => todo!(),
        BrowserSetup::None => debug!("No browser setup selected"),
    }
}

fn install_firefox() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "browser =.*",
            "browser = \"firefox\";",
        ),
        "Set Firefox",
    );
}