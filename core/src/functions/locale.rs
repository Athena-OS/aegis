use shared::args::{is_arch, is_nix};
use shared::exec::exec_archchroot;
use shared::files;
use shared::keyboard;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;

pub fn set_timezone(timezone: &str) {
    if !is_nix() {
        exec_eval(
            exec_archchroot(
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
            exec_archchroot("hwclock", vec!["--systohc".to_string()]),
            "Set system clock",
        );
    } else {
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/hosts/locale/default.nix",
                "Europe/Zurich",
                timezone,
            ),
            "Set Timezone",
        );        
    }
}

pub fn set_locale(locale: String) {
    if !is_nix() {
        files::create_file("/mnt/etc/locale.conf");
        files_eval(
            files::append_file("/mnt/etc/locale.conf", "LANG=en_US.UTF-8"),
            "edit locale.conf",
        );
        for i in (0..locale.split(' ').count()).step_by(2) {
            if is_arch() {
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
            }
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
        if is_arch(){
            exec_eval(exec_archchroot("locale-gen", vec![]), "generate locales");
        }
    } else {
        // Split the string into words using whitespace as delimiters and take only the first part
        let locale_part = locale.split_whitespace().next().unwrap_or("en_US.UTF-8");
        
        // Use only the extracted part of the locale in the sed_file call. Nix needs only the extracted part (i.e., en_US.UTF-8)
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/hosts/locale/default.nix",
                "en_US.UTF-8",
                locale_part,
            ),
            "Set Locale",
        );        
    }
}

pub fn set_keyboard(user_choice_or_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let km = keyboard::resolve(user_choice_or_id)
        .unwrap_or_else(|| keyboard::BY_ID["us"]); // safe fallback

    if !is_nix() {
        // Console
        files::create_file("/mnt/etc/vconsole.conf");
        files_eval(
            files::append_file("/mnt/etc/vconsole.conf", &format!("KEYMAP={}\n", km.console)),
            "set keyboard layout for virtual console",
        );
        files_eval(
            files::append_file("/mnt/etc/vconsole.conf", "FONT=ter-v24n\n"),
            "set console font",
        );

        // X11
        files_eval(files::create_directory("/mnt/etc/X11/xorg.conf.d"), "create /mnt/etc/X11/xorg.conf.d directory");
        let mut conf = String::new();
        conf.push_str(r#"
Section "InputClass"
    Identifier "system-keyboard"
    MatchIsKeyboard "on"
"#);
        conf.push_str(&format!("    Option \"XkbLayout\" \"{}\"\n", km.xkb_layout));
        if let Some(var) = km.xkb_variant {
            conf.push_str(&format!("    Option \"XkbVariant\" \"{var}\"\n"));
        }
        conf.push_str(r#"    Option "XkbModel" "pc105+inet"
    Option "XkbOptions" "terminate:ctrl_alt_bksp"
EndSection
"#);
        let mut file = std::fs::File::create("/mnt/etc/X11/xorg.conf.d/00-keyboard.conf")?;
        use std::io::Write;
        file.write_all(conf.as_bytes())?;
    } else {
        // NixOS branch (adjust to your files)
        files_eval(
            files::sed_file("/mnt/etc/nixos/hosts/locale/default.nix", "keyMap = \"us\";", &format!("keyMap = \"{}\";", km.console)),
            "Set Console Keyboard Layout (NixOS)",
        );
        files_eval(
            files::sed_file("/mnt/etc/nixos/hosts/locale/default.nix", "layout = \"us\";", &format!("layout = \"{}\";", km.xkb_layout)),
            "Set X11 Keyboard Layout (NixOS)",
        );
        if let Some(var) = km.xkb_variant {
            files_eval(
                files::sed_file("/mnt/etc/nixos/hosts/locale/default.nix", "variant = \"\";", &format!("variant = \"{var}\";")),
                "Set X11 Keyboard Variant (NixOS)",
            );
        }
    }

    Ok(())
}