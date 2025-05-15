use shared::exec::exec_archchroot;
use shared::files;
use shared::returncode_eval::exec_eval;
use shared::returncode_eval::files_eval;
use std::process::Command;

pub fn new_user(username: &str, hasroot: bool, password: &str, do_hash_pass: bool, shell: &str) {
    let shell: &str = shell;
    let mut _password = String::new();
    // Username cannot contain any space
    let sanitized_username = username.replace(' ', "");
    if do_hash_pass {
        let hashed_pass = hash_pass(password).stdout;
        _password = String::from_utf8_lossy(&hashed_pass).into_owned();
    }
    else {
        _password = password.to_string();
    }
    let shell_path = match shell {
        "bash" => "/bin/bash",
        "csh" => "/usr/bin/csh",
        "fish" => "/usr/bin/fish",
        "tcsh" => "/usr/bin/tcsh",
        "zsh" => "/usr/bin/zsh",
        &_ => "/usr/bin/bash",
    };
    exec_eval(
        exec_archchroot(
            "useradd",
            vec![
                String::from("-m"),
                String::from("-s"),
                String::from(shell_path),
                String::from("-p"),
                format!("'{}'", _password.replace('\n', "")),
                sanitized_username.clone(),
            ],
        ),
        format!("Create user {}", sanitized_username).as_str(),
    );
    if hasroot {
        exec_eval(
            exec_archchroot(
                "usermod",
                vec![
                    String::from("-aG"),
                    String::from("wheel,rfkill,sys,lp,input"),
                    sanitized_username.clone(),
                ],
            ),
            format!("Add user {} to wheel group", sanitized_username).as_str(),
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
        files::create_file(&format!("/mnt/var/lib/AccountsService/users/{}", sanitized_username));
        files_eval(
            files::append_file(
                &format!("/mnt/var/lib/AccountsService/users/{}", sanitized_username),
                "[User]\nSession=gnome-xorg\nIcon=/usr/share/pixmaps/faces/hackmyavatar.jpg",
            ),
            format!("Populate AccountsService user file for {}", sanitized_username).as_str(),
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
        exec_archchroot(
            "usermod",
            vec![
                String::from("--password"),
                format!("'{}'", root_pass.replace('\n', "")),
                String::from("root"),
            ],
        ),
        "set root password",
    );
}
