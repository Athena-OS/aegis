use log::{error};
use std::process::exit;

pub fn crash<S: AsRef<str>>(a: S, b: i32) -> ! {
    error!("{}", a.as_ref());
    exit(b);
}
