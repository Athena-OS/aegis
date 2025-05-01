use std::process::{Command, ExitStatus};
use std::io::{self, ErrorKind};
use std::path::Path;

const CHROOT_DIR: &str = "/mnt";

/// Bind mounts a host directory into the chroot.
fn bind_mount(source: &str, target: &str) -> io::Result<()> {
    Command::new("mount")
        .args(["--bind", source, target])
        .status()
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::new(ErrorKind::Other, format!("Failed to mount {}", target)))
            }
        })
}

/// Unmounts a directory inside the chroot.
fn unmount(target: &str) -> io::Result<()> {
    Command::new("umount")
        .arg(target)
        .status()
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::new(ErrorKind::Other, format!("Failed to unmount {}", target)))
            }
        })
}

/// Prepares /proc, /sys, /dev, /run in chroot.
fn setup_chroot_env() -> io::Result<()> {
    for dir in ["proc", "sys", "dev", "run"] {
        let mountpoint = format!("{}/{}", CHROOT_DIR, dir);
        if !Path::new(&mountpoint).exists() {
            std::fs::create_dir_all(&mountpoint)?;
        }
        bind_mount(&format!("/{}", dir), &mountpoint)?;
    }
    Ok(())
}

/// Cleans up the mounted filesystems in the chroot.
fn cleanup_chroot_env() -> io::Result<()> {
    // Unmount in reverse order to avoid dependency issues
    for dir in ["run", "dev", "sys", "proc"] {
        let mountpoint = format!("{}/{}", CHROOT_DIR, dir);
        unmount(&mountpoint)?;
    }
    Ok(())
}

/// Executes a command inside the chroot environment.
pub fn exec_chroot(command: &str, args: Vec<String>) -> io::Result<ExitStatus> {
    setup_chroot_env()?;

    let result = Command::new("chroot")
        .arg(CHROOT_DIR)
        .arg(command)
        .args(args)
        .status();

    // Always attempt cleanup, even if command fails
    if let Err(e) = cleanup_chroot_env() {
        eprintln!("Warning: Failed to clean up chroot mounts: {}", e);
    }

    result
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