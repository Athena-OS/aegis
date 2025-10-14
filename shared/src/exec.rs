use log::error;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::io::{self, BufRead, BufReader, Read};

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
                format!("Failed to mount {source} to {target}"),
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
                format!("Failed to unmount {target}"),
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
                format!("Failed to mount {source} to {target}"),
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
                format!("Failed to unmount {target}"),
            ));
        }
    }

    Ok(())
}

/// Executes a command inside the chroot environment.
pub fn exec_chroot(command: &str, args: Vec<String>) -> io::Result<()> {
    mount_chroot_base().expect("Failed to mount chroot filesystems");

    // capture stderr so it appears in your final error line
    let out = Command::new("chroot")
        .arg("/mnt")
        .arg(command)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    // Always try cleanup
    if let Err(e) = unmount_chroot_base() {
        error!("Warning: Failed to clean up chroot mounts: {e}");
    }

    let out = out?; // propagate spawn errors

    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(io::Error::other(
            if err.is_empty() {
                format!("chroot /mnt {command} {:?} exited with {}", args, out.status)
            } else {
                format!("chroot /mnt {command} {args:?} failed: {err}")
            },
        ))
    }
}

pub fn exec_archchroot(cmd: &str, args: Vec<String>) -> io::Result<()> {
    // call arch-chroot directly to avoid shell quoting issues
    let out = Command::new("arch-chroot")
        .arg("/mnt")
        .arg(cmd)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())  // or inherit/log if you prefer
        .stderr(Stdio::piped()) // capture for error message
        .output()?;

    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(io::Error::other(
            if err.is_empty() {
                format!("arch-chroot /mnt {cmd} {:?} exited with {}", args, out.status)
            } else {
                format!("arch-chroot /mnt {cmd} {args:?} failed: {err}")
            },
        ))
    }
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
        error!("System activation failed with exit code: {:?}", activate_status.code());
    }

    // Second: enter bash inside the new system
    let result = Command::new("chroot")
        .arg("/mnt")
        .arg(format!("/run/current-system/sw/bin/{} {}", command, args.join(" ")).as_str())
        .status();

    // Always attempt cleanup
    if let Err(e) = unmount_nixroot_base() {
        error!("Warning: Failed to clean up chroot mounts: {e}");
    }

    result
}

pub fn exec(command: &str, args: Vec<String>) -> io::Result<()> {
    let mut child = Command::new(command)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped()) // we'll log it via info!()
        .stderr(Stdio::piped()) // we'll capture it for error reporting
        .spawn()?;

    // --- log stdout line-by-line via info!() ---
    let mut stdout = child.stdout.take().expect("piped stdout");
    let stdout_handle = std::thread::spawn(move || -> io::Result<()> {
        let mut reader = BufReader::new(&mut stdout);
        let mut line = Vec::<u8>::new();
        loop {
            line.clear();
            let n = reader.read_until(b'\n', &mut line)?;
            if n == 0 { break; }
            let text = String::from_utf8_lossy(&line).trim_end_matches(&['\r','\n'][..]).to_string();
            log::info!("{text}");
        }
        Ok(())
    });

    // --- capture stderr fully (no live print) ---
    let mut stderr = child.stderr.take().expect("piped stderr");
    let stderr_handle = std::thread::spawn(move || -> io::Result<Vec<u8>> {
        let mut v = Vec::new();
        stderr.read_to_end(&mut v)?;
        Ok(v)
    });

    let status = child.wait()?;
    stdout_handle.join().unwrap()?;              // propagate stdout I/O errors
    let stderr_buf = stderr_handle.join().unwrap()?; // captured stderr

    if status.success() {
        Ok(())
    } else {
        let err_text = String::from_utf8_lossy(&stderr_buf).trim().to_string();
        let msg = if err_text.is_empty() {
            format!("{command} {args:?} exited with {status}")
        } else {
            format!("{command} {args:?} failed: {err_text}")
        };
        Err(io::Error::other(msg))
    }
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
) -> io::Result<()> {
    let out = Command::new(command)
        .args(&args)
        .current_dir(workdir)
        .stdin(Stdio::null())
        .output()?; // captures stdout/stderr

    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(io::Error::other(
            if err.is_empty() {
                format!("{command} {:?} exited with {}", args, out.status)
            } else {
                format!("{command} {args:?} failed: {err}")
            },
        ))
    }
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