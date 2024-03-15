use crate::internal::install::install;
use shared::args::PackageManager;
use shared::exec::exec_chroot;
use shared::files;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;
use std::process::Command;

pub fn new_user(username: &str, hasroot: bool, password: &str, do_hash_pass: bool, shell: &str) {
    let shell: &str = shell;
    if do_hash_pass {
        let hashed_pass = &*hash_pass(password).stdout;
        let _password = match std::str::from_utf8(hashed_pass) {
            Ok(v) => v,
            Err(e) => panic!("Failed to hash password, invalid UTF-8 sequence {}", e),
        };
    }
    let shell_to_install = match shell {
        "bash" => "bash",
        "csh" => "tcsh",
        "fish" => "fish",
        "tcsh" => "tcsh",
        "zsh" => "zsh",
        &_ => "bash",
    };
    install(PackageManager::Pacman, vec![shell_to_install]);
    let shell_path = match shell {
        "bash" => "/bin/bash",
        "csh" => "/usr/bin/csh",
        "fish" => "/usr/bin/fish",
        "tcsh" => "/usr/bin/tcsh",
        "zsh" => "/usr/bin/zsh",
        &_ => "/usr/bin/bash",
    };
    exec_eval(
        exec_chroot(
            "useradd",
            vec![
                String::from("-m"),
                String::from("-s"),
                String::from(shell_path),
                String::from("-p"),
                String::from(password).replace('\n', ""),
                String::from(username),
            ],
        ),
        format!("Create user {}", username).as_str(),
    );
    if hasroot {
        exec_eval(
            exec_chroot(
                "usermod",
                vec![
                    String::from("-aG"),
                    String::from("wheel,rfkill,sys,lp,input"),
                    String::from(username),
                ],
            ),
            format!("Add user {} to wheel group", username).as_str(),
        );
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
        files::create_file(&format!("/mnt/var/lib/AccountsService/users/{}", username));
        files_eval(
            files::append_file(
                &format!("/mnt/var/lib/AccountsService/users/{}", username),
                "[User]\nSession=gnome-xorg\nIcon=/usr/share/pixmaps/faces/hackmyavatar.jpg",
            ),
            format!("Populate AccountsService user file for {}", username).as_str(),
        )
    }
}

pub fn hash_pass(password: &str) -> std::process::Output {
    let output = Command::new("openssl")
        .args(["passwd", "-6", password])
        .output()
        .expect("Failed to hash password");

    output
}

pub fn root_pass(root_pass: &str) {
    exec_eval(
        exec_chroot(
            "bash",
            vec![
                String::from("-c"),
                format!(r#"'usermod --password {root_pass} root'"#),
            ],
        ),
        "set root password",
    );
}