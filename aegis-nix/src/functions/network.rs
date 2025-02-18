use shared::files;
use shared::returncode_eval::files_eval;

pub fn set_hostname(hostname: &str) {
    // Remove spaces from the hostname string because hostname cannot contain any space
    let sanitized_hostname = hostname.replace(' ', "");

    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "  hostname =.*",
            &(format!("  hostname = \"{}\";", sanitized_hostname)),
        ),
        "Set Hostname",
    );
}

pub fn enable_ipv6() {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/configuration.nix",
            "networking.enableIPv6 =.*",
            "networking.enableIPv6 = true;",
        ),
        "enable ipv6",
    );
}
