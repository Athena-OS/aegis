use log::debug;
use shared::args::{ExecMode, OnFail};
use shared::exec::exec;
use shared::returncode_eval::exec_eval;

pub fn enable_service(dm: &str) {
    debug!("Enabling {dm}");
    exec_eval(
        exec(ExecMode::Chroot { root: "/mnt" }, "systemctl", vec![String::from("enable"), String::from(dm)], OnFail::Error),
        format!("Enable {dm}").as_str(),
    );
}

pub fn disable_service(dm: &str) {
    debug!("Disabling {dm}");
    exec_eval(
        exec(ExecMode::Chroot { root: "/mnt" }, "systemctl", vec![String::from("disable"), String::from(dm)], OnFail::Error),
        format!("Disable {dm}").as_str(),
    );
}