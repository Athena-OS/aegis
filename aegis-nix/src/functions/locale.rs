use shared::files;
use shared::returncode_eval::files_eval;

pub fn set_timezone(timezone: &str) {
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/hosts/locale/default.nix",
            "Europe/Zurich",
            timezone,
        ),
        "Set Timezone",
    );
}

pub fn set_locale(locale: String) {
    // Split the string into words using whitespace as delimiters and take only the first part
    let locale_part = locale.split_whitespace().next().unwrap_or("en_US.UTF-8");

    // Use only the extracted part of the locale in the sed_file call. Nix needs only the extracted part (i.e., en_US.UTF-8)
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/hosts/locale/default.nix",
            "en_US.UTF-8",
            &locale_part,
        ),
        "Set Locale",
    );
}

pub fn set_keyboard(virtkeyboard: &str, x11keyboard: &str) {
    // Setting keyboard layout for virtual console (TTY)
    // and keyboard layout for X (GUI) environment (note: Wayland keyboard layout is managed by the used compositors)
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/hosts/locale/default.nix",
            "keyMap = \"us\";",
            &(format!("keyMap = \"{}\";", virtkeyboard)),
        ),
        "Set Console Keyboard Layout",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/nixos/hosts/locale/default.nix",
            "layout = \"us\";",
            &(format!("layout = \"{}\";", x11keyboard)),
        ),
        "Set x11 Keyboard Layout",
    );
}
