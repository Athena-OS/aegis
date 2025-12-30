use log::{error};
use std::process::exit;

fn fmt_arg(a: &str) -> String {
    let needs_quotes = a.chars().any(|c| c.is_whitespace() || "\"'\\$`!(){}[]<>|&;*?".contains(c));
    if !needs_quotes {
        return a.to_string();
    }
    let escaped = a.replace('\'', r"'\''");
    format!("'{escaped}'")
}

pub fn fmt_cmdline(cmd: &str, args: &[String]) -> String {
    let mut s = String::new();
    s.push_str(&fmt_arg(cmd));
    for a in args {
        s.push(' ');
        s.push_str(&fmt_arg(a));
    }
    s
}

pub fn crash<S: AsRef<str>>(a: S, b: i32) -> ! {
    error!("{}", a.as_ref());
    exit(b);
}
