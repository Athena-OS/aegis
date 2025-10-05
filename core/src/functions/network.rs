use log::info;
use shared::args::is_nix;
use shared::files;
use shared::returncode_eval::files_eval;

pub fn set_hostname(hostname: &str) {
    info!("Setting hostname to {hostname}");
    if !is_nix() {
        files::create_file("/mnt/etc/hostname");
        files_eval(
            files::append_file("/mnt/etc/hostname", hostname),
            "set hostname",
        );
    } else {
        let sanitized_hostname = hostname.replace(' ', "");
        
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                "  hostname =.*",
                &(format!("  hostname = \"{sanitized_hostname}\";")),
            ),
            "Set Hostname",
        );        
    }
}

pub fn create_hosts() {
    files::create_file("/mnt/etc/hosts");
    files_eval(
        files::append_file("/mnt/etc/hosts", "127.0.0.1     localhost"),
        "create /etc/hosts",
    );
}
