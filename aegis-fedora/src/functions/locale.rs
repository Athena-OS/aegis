use std::fs::File;
use std::io::Write;
use shared::exec::exec_chroot;
use shared::files;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;

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
        /*
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
        */
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

pub fn set_keyboard(virtkeyboard: &str, x11keyboard: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Setting keyboard layout for virtual console (TTY)
    files::create_file("/mnt/etc/vconsole.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/vconsole.conf",
            format!("KEYMAP={}", virtkeyboard).as_str(),
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
    files_eval(files::create_directory("/mnt/etc/X11/xorg.conf.d"), "create /mnt/etc/X11/xorg.conf.d directory");
        let conf_content = format!(
        r#"# Written by systemd-localed(8), read by systemd-localed and Xorg. It's
# probably wise not to edit this file manually. Use localectl(1) to
# instruct systemd-localed to update it.
Section "InputClass"
        Identifier "system-keyboard"
        MatchIsKeyboard "on"
        Option "XkbLayout" "{}"
        Option "XkbModel" "pc105+inet"
        Option "XkbOptions" "terminate:ctrl_alt_bksp"
EndSection
"#,
        x11keyboard
    );
    let mut file = File::create("/mnt/etc/X11/xorg.conf.d/00-keyboard.conf")?;
    file.write_all(conf_content.as_bytes())?;

    Ok(())
}
