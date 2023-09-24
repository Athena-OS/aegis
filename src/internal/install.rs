use crate::functions::partition::umount;
use crate::internal::*;
use log::{error, info};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

pub fn install(pkgs: Vec<&str>) {
    let mut pacstrap_cmd = Command::new("pacstrap")
        .arg("/mnt")
        .args(&pkgs)
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()
        .expect("Failed to start pacstrap");

    let stdout_handle = pacstrap_cmd.stdout.take().unwrap();
    let stderr_handle = pacstrap_cmd.stderr.take().unwrap();

    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout_handle);
        for line in reader.lines() {
            let line = line.expect("Failed to read stdout");
            info!("{}", line);
        }
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr_handle);
        for line in reader.lines() {
            let line = line.expect("Failed to read stderr");
            error!("pacstrap stderr: {}", line);
        }
    });

    let exit_status = pacstrap_cmd.wait().expect("Failed to wait for pacstrap");

    // Wait for the stdout and stderr threads to finish
    stdout_thread.join().expect("stdout thread panicked");
    stderr_thread.join().expect("stderr thread panicked");

    if !exit_status.success() {
        // Handle the error here, e.g., by logging it
        error!("pacstrap failed with exit code: {}", exit_status.code().unwrap_or(-1));
    }

    umount("/mnt/dev");
}
