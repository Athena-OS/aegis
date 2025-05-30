use shared::debug;
use shared::exec::exec_chroot;
use shared::returncode_eval::exec_eval;

pub fn enable_service(dm: &str) {
    debug!("Enabling {}", dm);
    exec_eval(
        exec_chroot("systemctl", vec![String::from("enable"), String::from(dm)]),
        format!("Enable {}", dm).as_str(),
    );
}

pub fn disable_service(dm: &str) {
    debug!("Disabling {}", dm);
    exec_eval(
        exec_chroot("systemctl", vec![String::from("disable"), String::from(dm)]),
        format!("Disable {}", dm).as_str(),
    );
}