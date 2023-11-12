use crate::internal::exec::*;
use crate::internal::*;

pub fn set_timezone(timezone: &str) {
    exec_eval(
        exec_chroot(
            "ln",
            vec![
                "-sf".to_string(),
                format!("/usr/share/zoneinfo/{}", timezone),
                "/etc/localtime".to_string(),
            ],
        ),
        "Set timezone",
    );
    exec_eval(
        exec_chroot("hwclock", vec!["--systohc".to_string()]),
        "Set system clock",
    );
}

pub fn set_locale(locale: String) {
    files::create_file("/mnt/etc/locale.conf");
    files_eval(
        files::append_file("/mnt/etc/locale.conf", "LANG=en_US.UTF-8"),
        "edit locale.conf",
    );
    for i in (0..locale.split(' ').count()).step_by(2) {
        files_eval(
            files::append_file(
                "/mnt/etc/locale.gen",
                &format!(
                    "{} {}\n",
                    locale.split(' ').collect::<Vec<&str>>()[i],
                    locale.split(' ').collect::<Vec<&str>>()[i + 1]
                ),
            ),
            "add locales to locale.gen",
        );
        if locale.split(' ').collect::<Vec<&str>>()[i] != "en_US.UTF-8" {
            files_eval(
                files::sed_file(
                    "/mnt/etc/locale.conf",
                    "en_US.UTF-8",
                    locale.split(' ').collect::<Vec<&str>>()[i],
                ),
                format!(
                    "Set locale {} in /etc/locale.conf",
                    locale.split(' ').collect::<Vec<&str>>()[i]
                )
                .as_str(),
            );
        }
    }
    exec_eval(exec_chroot("locale-gen", vec![]), "generate locales");
}

pub fn set_keyboard(keyboard: &str) {
    // Setting keyboard layout for virtual console (TTY)
    files::create_file("/mnt/etc/vconsole.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/vconsole.conf",
            format!("KEYMAP={}", keyboard).as_str(),
        ),
        "set keyboard layout for virtual console",
    );
    files_eval(
        files::append_file(
            "/mnt/etc/vconsole.conf",
            "FONT=ter-v24n",
        ),
        "set console font",
    );
    // Setting keyboard layout for X (GUI) environment (note: Wayland keyboard layout is managed by the used compositors)
    files_eval(files::create_directory("/mnt/etc/X11/xorg.conf.d"), "create /mnt/etc/X11/xorg.conf.d");
    files::copy_file("/etc/X11/xorg.conf.d/00-keyboard.conf", "/mnt/etc/X11/xorg.conf.d/00-keyboard.conf");
    files_eval(
        files::sed_file(
            "/mnt/etc/X11/xorg.conf.d/00-keyboard.conf",
            "\"us\"",
            format!("\"{}\"", keyboard).as_str(),
        ),
        "set keyboard layout for X environment",
    );
}
