use std::process::{Command, ExitStatus, Output};
use std::io;

pub fn mount_chroot_base() -> io::Result<()> {
    let mounts = vec![
        ("proc", "/mnt/proc", "proc"),
        ("sysfs", "/mnt/sys", "sysfs"),
        ("/dev", "/mnt/dev", "bind"),
        ("/run", "/mnt/run", "bind"),
        ("/sys/fs/selinux", "/mnt/sys/fs/selinux", "bind"),
    ];

    for (source, target, fstype) in mounts {
        std::fs::create_dir_all(target)?;

        let status = if fstype == "bind" {
            Command::new("mount")
                .args(["--bind", source, target])
                .status()?
        } else {
            Command::new("mount")
                .args(["-t", fstype, source, target])
                .status()?
        };

        if !status.success() {
            return Err(io::Error::other(
                format!("Failed to mount {} to {}", source, target),
            ));
        }
    }

    Ok(())
}

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
            return Err(io::Error::other(
                format!("Failed to unmount {}", target),
            ));
        }
    }

    Ok(())
}

fn mount_nixroot_base() -> io::Result<()> {
    let mounts = vec![
        ("/proc", "/mnt/proc", "bind"),
        ("/sys", "/mnt/sys", "bind"),
        ("/dev", "/mnt/dev", "bind"),
    ];

    for (source, target, fstype) in mounts {
        std::fs::create_dir_all(target)?;

        let status = if fstype == "bind" {
            Command::new("mount")
                .args(["-o", "bind", source, target])
                .status()?
        } else {
            Command::new("mount")
                .args(["-t", fstype, source, target])
                .status()?
        };

        if !status.success() {
            return Err(io::Error::other(
                format!("Failed to mount {} to {}", source, target),
            ));
        }
    }

    Ok(())
}

fn unmount_nixroot_base() -> io::Result<()> {
    for target in [
        "/mnt/dev",
        "/mnt/sys",
        "/mnt/proc",
    ] {
        let status = Command::new("umount").arg(target).status()?;

        if !status.success() {
            return Err(io::Error::other(
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

pub fn exec_archchroot(
    command: &str,
    args: Vec<String>,
) -> Result<ExitStatus, std::io::Error> {
    let returncode = Command::new("bash")
        .args([
            "-c",
            format!("arch-chroot /mnt {} {}", command, args.join(" ")).as_str(),
        ])
        .status();
    returncode
}

/// Executes a command inside the nix chroot environment.
pub fn exec_nixroot(
    command: &str,
    args: Vec<String>,
) -> Result<ExitStatus, std::io::Error> {
    mount_nixroot_base().expect("Failed to mount chroot filesystems");

    // First: run system activation
    let activate_status = Command::new("chroot")
        .arg("/mnt")
        .arg("/nix/var/nix/profiles/system/activate")
        .status()?;

    if !activate_status.success() {
        eprintln!("System activation failed with exit code: {:?}", activate_status.code());
    }

    // Second: enter bash inside the new system
    let result = Command::new("chroot")
        .arg("/mnt")
        .arg(format!("/run/current-system/sw/bin/{} {}", command, args.join(" ")).as_str())
        .status();

    // Always attempt cleanup
    if let Err(e) = unmount_nixroot_base() {
        eprintln!("Warning: Failed to clean up chroot mounts: {}", e);
    }

    result
}

pub fn exec(command: &str, args: Vec<String>) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new(command).args(args).status();
    returncode
}

pub fn exec_output(command: &str, args: Vec<String>) -> Result<Output, io::Error> {
    let output = Command::new(command)
        .args(args)
        .output()?; // propagates std::io::Error if the command fails to run

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(err_msg.to_string()));
    }

    Ok(output)
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