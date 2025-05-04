use shared::exec::exec;
use shared::exec::exec_chroot;
use shared::returncode_eval::exec_eval;
use std::path::Path;

pub fn selinux_enabled() -> bool {
    Path::new("/sys/fs/selinux/enforce").exists()
}

pub fn set_selinux_mode(mode: &str) {
    exec_eval(
        exec(
            "setenforce",
            vec![
                mode.to_string(),
            ],
        ),
        &format!("SELinux mode to {}", mode),
    );
}

pub fn set_security_context() {
    exec_eval(
        exec_chroot(
            "restorecon",
            vec![
                String::from("-Rv"),
                String::from("/"),
            ],
        ),
        "Set SELinux security context",
    );
}

/*
pub fn secure_password_config() {
    files_eval(
        files::sed_file(
            "/mnt/etc/login.defs",
            "PASS_MAX_DAYS	99999",
            "PASS_MAX_DAYS	365",
        ),
        "Set the password expiration to 365 days",
    );
}

pub fn secure_ssh_config() {
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#Port.*",
            "Port 2222",
        ),
        "Setting SSH port to 2222",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#PermitRootLogin.*",
            "PermitRootLogin no",
        ),
        "Prevent root login",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#PubkeyAuthentication.*",
            "PubkeyAuthentication yes",
        ),
        "Allow public key authentication",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#PasswordAuthentication.*",
            "PasswordAuthentication no",
        ),
        "Prevent password authentication",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#PermitEmptyPasswords.*",
            "PermitEmptyPasswords no",
        ),
        "Prevent password authentication",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#IgnoreRhosts.*",
            "IgnoreRhosts yes",
        ),
        "Prevent remote hosts to be used in authentication",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#LoginGraceTime.*",
            "LoginGraceTime 30",
        ),
        "Set a secure login grace time",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#MaxAuthTries.*",
            "MaxAuthTries 4",
        ),
        "Set a maximum number of permitted authentication attempts per connection",
    );
    files_eval(
        files::sed_file(
            "/mnt/etc/ssh/sshd_config",
            "#HostbasedAuthentication.*",
            "HostbasedAuthentication no",
        ),
        "Prevent authentication via .rhosts file",
    );
    files_eval(
        files::append_file("/mnt/etc/ssh/sshd_config", "Protocol 2"),
        "Set SSH protocol 2",
    );
}
*/