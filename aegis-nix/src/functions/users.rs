use shared::{error, info};
use shared::exec::exec_output;
use shared::files;
use shared::returncode_eval::exec_eval_result;

pub fn new_user(username: &str, password: &str, do_hash_pass: bool) {
    let mut _password = String::new();
    let config_path = "/mnt/etc/nixos/configuration.nix";
    // Username cannot contain any space
    let sanitized_username = username.replace(' ', "");
    
    let user_line = format!("  username = \"{}\";", sanitized_username);
    match files::replace_line_in_file(config_path, "  username = \"", &user_line) {
        Ok(_) => {
            info!("Set username");
        }
        Err(e) => {
            error!("Set username ERROR: {}", e);
        }
    }
    
    if do_hash_pass {
        let hashed_pass = hash_pass(password).stdout;
        _password = String::from_utf8_lossy(&hashed_pass).into_owned();
    }
    else {
        _password = password.to_string();
    }
    
    let hash_line = format!("  hashed = \"{}\";", _password);
    match files::replace_line_in_file(config_path, "  hashed = \"", &hash_line) {
        Ok(_) => {
            info!("Set user password hash");
        }
        Err(e) => {
            error!("Set user password hash ERROR: {}", e);
        }
    }
}

pub fn hash_pass(password: &str) -> std::process::Output {
    exec_eval_result(
        exec_output(
            "openssl",
            vec![
                String::from("passwd"),
                String::from("-6"),
                password.to_string()
            ]
        ),
        "Compute the password hash",
    )
}

pub fn root_pass(root_pass: &str) {
    let config_path = "/mnt/etc/nixos/configuration.nix";
    
    let hash_line = format!("  hashedRoot = \"{}\";", root_pass);
    match files::replace_line_in_file(config_path, "  hashedRoot = \"", &hash_line) {
        Ok(_) => {
            info!("Set root password hash");
        }
        Err(e) => {
            error!("Set root password hash ERROR: {}", e);
        }
    }
}
