use shared::args::PackageManager;
use shared::{debug, error, info};
use shared::exec;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

fn selinux_enabled() -> bool {
    Path::new("/sys/fs/selinux/enforce").exists()
}

fn set_selinux_mode(mode: &str) -> std::io::Result<ExitStatus> {
    Command::new("setenforce")
        .arg(mode)
        .status()
}

pub fn install(pkgmanager: PackageManager, pkgs: Vec<&str>, excluded_pkgs: Option<Vec<&str>>) {
    let mut retry = Arc::new(Mutex::new(true));
    let mut retry_counter = 0;

    while *retry.lock().unwrap() && retry_counter < 15 {
        retry = Arc::new(Mutex::new(false));

        let mut pkgmanager_cmd = Command::new("true")
            .spawn()
            .expect("Failed to initialize by 'true'");

        if selinux_enabled() {
            if let Err(err) = set_selinux_mode("0") {
                eprintln!("Warning: Could not set SELinux to permissive: {}", err);
            }
        }

        match pkgmanager {
            PackageManager::Dnf => {
                exec::mount_chroot_base().expect("Failed to mount chroot filesystems");

                Command::new("dnf")
                    .arg("makecache")
                    .arg("--refresh")
                    .status()
                    .expect("Failed to refresh dnf cache");

                let mut cmd = Command::new("dnf");
                cmd.arg("--installroot=/mnt")
                    .arg("--setopt=install_weak_deps=False")
                    .arg("--use-host-config")
                    .arg("install")
                    .arg("-y")
                    .args(&pkgs);

                if let Some(ref excludes) = excluded_pkgs {
                    if !excludes.is_empty() {
                        cmd.arg(format!("--exclude={}", excludes.join(",")));
                    }
                }

                pkgmanager_cmd = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start dnf");
            }

            PackageManager::RpmOSTree => {
                Command::new("chroot")
                    .arg("/mnt")
                    .arg("rpm-ostree")
                    .arg("refresh-md")
                    .status()
                    .expect("Failed to refresh rpm-ostree metadata");

                pkgmanager_cmd = Command::new("chroot")
                    .arg("/mnt")
                    .arg("rpm-ostree")
                    .arg("install")
                    .args(&pkgs)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start rpm-ostree install in chroot");
            }

            PackageManager::None => debug!("No package manager selected"),
        };

        let stdout_handle = pkgmanager_cmd.stdout.take().unwrap();
        let stderr_handle = pkgmanager_cmd.stderr.take().unwrap();

        let stdout_thread = spawn_log_thread(BufReader::new(stdout_handle));
        let stderr_thread = spawn_log_thread(BufReader::new(stderr_handle));

        let exit_status = pkgmanager_cmd.wait().expect("Failed to wait for the package manager");

        stdout_thread.join().expect("stdout thread panicked");
        stderr_thread.join().expect("stderr thread panicked");

        if !exit_status.success() {
            error!("The package manager failed with exit code: {}", exit_status.code().unwrap_or(-1));
        }

        if let Err(e) = exec::unmount_chroot_base() {
            eprintln!("Warning: Failed to unmount chroot base: {}", e);
        }

        if selinux_enabled() {
            if let Err(err) = set_selinux_mode("1") {
                eprintln!("Warning: Failed to re-enable SELinux: {}", err);
            }
        }

        retry_counter += 1;
    }
}

// Helper function to handle both stdout and stderr
fn spawn_log_thread<R: BufRead + Send + 'static>(
    reader_handle: R,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(reader_handle);
        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            info!("{}", line);
        }
    })
}