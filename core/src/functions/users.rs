use log::{error, info};
use shared::args::{ExecMode, OnFail, is_nix};
use shared::exec::exec;
use shared::files;
use shared::returncode_eval::{exec_eval, files_eval};

pub fn new_user(username: &str, password: &str, groups: &[String], shell: &str) {
    let shell: &str = shell;
    let mut _password = String::new();
    // Username cannot contain any space
    let sanitized_username = username.replace(' ', "");
    
    _password = password.to_string(); // hashed

    if !is_nix() {
        let shell_path = match shell {
            "bash" => "/bin/bash",
            "csh" => "/usr/bin/csh",
            "fish" => "/usr/bin/fish",
            "tcsh" => "/usr/bin/tcsh",
            "zsh" => "/usr/bin/zsh",
            &_ => "/usr/bin/bash",
        };
        exec_eval(
            exec(
                ExecMode::Chroot { root: "/mnt" },
                "useradd",
                vec![
                    String::from("-m"),
                    String::from("-s"),
                    String::from(shell_path),
                    String::from("-p"),
                    format!("{}", _password.replace('\n', "")),
                    sanitized_username.clone(),
                ],
                OnFail::Error,
            ),
            format!("Create user {sanitized_username}").as_str(),
        );
        if !groups.is_empty() {
            exec_eval(
                exec(
                    ExecMode::Chroot { root: "/mnt" },
                    "usermod",
                    vec![
                        String::from("-aG"),
                        groups.join(","),
                        sanitized_username.clone(),
                    ],
                    OnFail::Error,
                ),
                format!("Add user {sanitized_username} to specified groups").as_str(),
            );
        }
        files_eval(
            files::sed_file(
                "/mnt/etc/sudoers",
                "# %wheel ALL=\\(ALL:ALL\\) ALL",
                "%wheel ALL=(ALL:ALL) ALL",
            ),
            "Add wheel group to sudoers",
        );
        /*files_eval( // pwfeedback is used to show psw asterisks during sudo
            files::append_file("/mnt/etc/sudoers", "\nDefaults pwfeedback\n"),
            "Add pwfeedback to sudoers",
        );*/
        files_eval(
            files::create_directory("/mnt/var/lib/AccountsService/users/"),
            "Create /mnt/var/lib/AccountsService",
        );
        files::create_file(&format!("/mnt/var/lib/AccountsService/users/{sanitized_username}"));
        files_eval(
            files::append_file(
                &format!("/mnt/var/lib/AccountsService/users/{sanitized_username}"),
                "[User]\nSession=gnome-xorg\nIcon=/usr/share/pixmaps/faces/hackmyavatar.jpg",
            ),
            format!("Populate AccountsService user file for {sanitized_username}").as_str(),
        )
    } else {
        let user_line = format!("  username = \"{sanitized_username}\";");
        let config_path = "/mnt/etc/nixos/configuration.nix";
        match files::replace_line_in_file(config_path, "  username = \"", &user_line) {
            Ok(_) => {
                info!("Set username");
            }
            Err(e) => {
                error!("Set username ERROR: {e}");
            }
        }

        let hash_line = format!("  hashed = \"{_password}\";");
        match files::replace_line_in_file(config_path, "  hashed = \"", &hash_line) {
            Ok(_) => {
                info!("Set user password hash");
            }
            Err(e) => {
                error!("Set user password hash ERROR: {e}");
            }
        }

        let groups_nix = groups
            .iter()
            .map(|g| format!(r#""{g}""#))
            .collect::<Vec<_>>()
            .join(" ");
            
        // Replace the whole extraGroups assignment, preserving indentation
        files_eval(
            files::sed_file(
                "/mnt/etc/nixos/configuration.nix",
                r"(?ms)^(\s*)extraGroups\s*=\s*\[.*?\];",
                &format!(r#"${{1}}extraGroups = [ {groups_nix} ];"#),
            ),
            "Set user extraGroups",
        );
    }
}

pub fn root_pass(root_pass: &str) {
    if !is_nix() {
        exec_eval(
            exec(
                ExecMode::Chroot { root: "/mnt" },
                "usermod",
                vec![
                    String::from("--password"),
                    format!("{}", root_pass.replace('\n', "")),
                    String::from("root"),
                ],
                OnFail::Error,
            ),
            "set root password",
        );
    } else {
        let config_path = "/mnt/etc/nixos/configuration.nix";
        
        let hash_line = format!("  hashedRoot = \"{root_pass}\";");
        match files::replace_line_in_file(config_path, "  hashedRoot = \"", &hash_line) {
            Ok(_) => {
                info!("Set root password hash");
            }
            Err(e) => {
                error!("Set root password hash ERROR: {e}");
            }
        }        
    }
}
