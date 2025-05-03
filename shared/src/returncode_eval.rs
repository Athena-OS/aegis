use crate::log::{info};
use crate::strings::crash;
use std::io;

pub fn exec_eval(
    return_code: std::result::Result<std::process::ExitStatus, std::io::Error>,
    logmsg: &str,
) {
    match &return_code {
        Ok(_) => {
            info!("{}", logmsg);
        }
        Err(e) => {
            crash(
                format!("{}  ERROR: {}", logmsg, e),
                return_code.unwrap_err().raw_os_error().unwrap(),
            );
        }
    }
}

pub fn exec_eval_result<T>(
    result: Result<T, io::Error>,
    logmsg: &str
) -> T {
    match result {
        Ok(val) => {
            info!("{}", logmsg);
            val
        }
        Err(e) => {
            crash(
                format!("{}  ERROR: {}", logmsg, e),
                e.raw_os_error().unwrap_or(1),
            );
        }
    }
}

pub fn files_eval(return_code: std::result::Result<(), std::io::Error>, logmsg: &str) {
    match &return_code {
        Ok(_) => {
            info!("{}", logmsg);
        }
        Err(e) => {
            info!("{} ERROR: {}", logmsg, e);
            crash(
                format!("{} ERROR: {}", logmsg, e),
                return_code.unwrap_err().raw_os_error().unwrap(),
            );
        }
    }
}
