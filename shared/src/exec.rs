use std::process::{Command, ExitStatus};
use std::io::{self, ErrorKind};

pub fn unmount_chroot_base() -> io::Result<()> {
    for target in [
        "/mnt/sys/fs/selinux",
        "/mnt/run",
        "/mnt/dev",
        "/mnt/sys",
        "/mnt/proc",
    ] {
        let status = Command::new("umount").arg(target).status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to unmount {}", target),
            ));
        }
    }

    Ok(())
}

/// Executes a command inside the chroot environment.
pub fn exec_chroot(command: &str, args: Vec<String>) -> io::Result<ExitStatus> {
    mount_chroot_base().expect("Failed to mount chroot filesystems");

    let result = Command::new("chroot")
        .arg("/mnt")
        .arg(command)
        .args(args)
        .status();

    // Always attempt cleanup, even if command fails
    if let Err(e) = unmount_chroot_base() {
        eprintln!("Warning: Failed to clean up chroot mounts: {}", e);
    }

    result
}

pub fn exec_chroot_capture(command: &str, args: Vec<String>) -> io::Result<String> {
    mount_chroot_base().expect("Failed to mount chroot filesystems");

    let output = Command::new("chroot")
        .arg("/mnt")
        .arg(command)
        .args(args)
        .output();

    if let Err(e) = unmount_chroot_base() {
        eprintln!("Warning: Failed to clean up chroot mounts: {}", e);
    }

    match output {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).trim().to_string()),
        Ok(out) => Err(io::Error::new(io::ErrorKind::Other, String::from_utf8_lossy(&out.stderr).to_string())),
        Err(e) => Err(e),
    }
}

pub fn exec(command: &str, args: Vec<String>) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new(command).args(args).status();
    returncode
}

pub fn exec_workdir(
    command: &str,
    workdir: &str,
    args: Vec<String>,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new(command)
        .args(args)
        .current_dir(workdir)
        .status();
    returncode
}

pub fn check_if_root() -> bool {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to execute command");

    if let Ok(euid_str) = String::from_utf8(output.stdout) {
        let euid: u32 = euid_str.trim().parse().unwrap_or(1);
        if euid != 0 {
            eprintln!("You must be root to perform this operation.");
            std::process::exit(1);
        }
        return true; // Return true if running as root
    }

    false // If there's an error, return false
}