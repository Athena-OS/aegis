use crate::strings::crash;
use log::{info};
use std::io;

pub fn exec_eval<T>(result: Result<T, std::io::Error>, logmsg: &str) -> T {
    match result {
        Ok(v) => {
            log::info!("{logmsg}");
            v
        }
        Err(e) => {
            let code = e.raw_os_error().unwrap_or(1);
            crate::strings::crash(format!("{e}"), code);
        }
    }
}

pub fn exec_eval_result<T>(
    result: Result<T, io::Error>,
    logmsg: &str
) -> T {
    match result {
        Ok(val) => {
            info!("{logmsg}");
            val
        }
        Err(e) => {
            crash(
                format!("{logmsg}  ERROR: {e}"),
                e.raw_os_error().unwrap_or(1),
            );
        }
    }
}

pub fn files_eval(return_code: std::result::Result<(), std::io::Error>, logmsg: &str) {
    match &return_code {
        Ok(_) => {
            info!("{logmsg}");
        }
        Err(e) => {
            info!("{logmsg} ERROR: {e}");
            crash(
                format!("{logmsg} ERROR: {e}"),
                return_code.unwrap_err().raw_os_error().unwrap(),
            );
        }
    }
}
