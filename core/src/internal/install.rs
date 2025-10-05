use log::{debug, error, info};
use regex::Regex;
use shared::args::{InstallMode, PackageManager};
use shared::exec;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn install(
    pkgmanager: PackageManager,
    pkgs: Vec<String>,
    mode: Option<InstallMode>, // Arch: pass None; Fedora: Some(…)
) -> i32 {
    let mode = mode.unwrap_or(InstallMode::Install);
    // retry loop is meaningful for Arch; Fedora will run once (retry never set)
    let mut retry = Arc::new(Mutex::new(true));
    let mut retry_counter = 0;
    let mut last_exit = 0;

    while *retry.lock().unwrap() && retry_counter < 15 {
        // reset flag for this iteration
        retry = Arc::new(Mutex::new(false));

        // placeholder child, replaced in match arms
        let mut child = Command::new("true")
            .spawn()
            .expect("Failed to initialize dummy command");

        // Arch arms set this to Some("pacstrap"/"pacman"), Fedora leaves it None
        let mut pkgmanager_name: Option<String> = None;

        match pkgmanager {
            // ---------- ARCH ----------
            PackageManager::Pacstrap => {
                pkgmanager_name = Some("pacstrap".to_string());
                child = Command::new("pacstrap")
                    .arg("/mnt")
                    .args(&pkgs)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start pacstrap");
            }
            PackageManager::Pacman => {
                pkgmanager_name = Some("pacman".to_string());
                child = Command::new("arch-chroot")
                    .arg("/mnt")
                    .arg("pacman")
                    .arg("-Syyu")
                    .arg("--needed")
                    .arg("--noconfirm")
                    .args(&pkgs)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start pacman");
            }

            // ---------- FEDORA ----------
            PackageManager::Dnf => {
                exec::mount_chroot_base().expect("Failed to mount chroot filesystems");

                // refresh cache (host config)
                Command::new("dnf")
                    .arg("makecache")
                    .arg("--refresh")
                    .status()
                    .expect("Failed to refresh dnf cache");

                let mut cmd = Command::new("dnf");
                cmd.arg("--installroot=/mnt")
                    .arg("--use-host-config");
                match mode {
                    InstallMode::Install => {
                        cmd.arg("install")
                            .arg("-y")
                            .arg("--setopt=install_weak_deps=False")
                            .args(&pkgs);
                    }
                    InstallMode::Remove => {
                        cmd.arg("remove").arg("-y").args(&pkgs);
                    }
                }
                child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start dnf");
            }
            PackageManager::RpmOSTree => {
                let mut cmd = Command::new("chroot");
                cmd.arg("/mnt").arg("rpm-ostree");
                match mode {
                    InstallMode::Install => cmd.arg("install").args(&pkgs),
                    InstallMode::Remove  => cmd.arg("override").arg("remove").args(&pkgs),
                };
                child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start rpm-ostree");
            }

            // ---------- NIXOS ----------
            PackageManager::Nix => {
                let install_nixos_args = "nixos-install --no-root-password --keep-going".to_string();
                let install_args = vec!["-p", "nixos-install-tools", "--run", &install_nixos_args];

                child = Command::new("nix-shell")
                    .args(&install_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start nixos-install (nix-shell).");
            }
            PackageManager::None => {
                debug!("No package manager selected");
            }
        };

        // take pipes
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Arch: pass Some(retry,pkgmanager_name) to enable mirror retry logic
        // Fedora and Nix: pass None,None to just stream logs
        let rflag = match pkgmanager {
            PackageManager::Pacstrap | PackageManager::Pacman => Some(Arc::clone(&retry)),
            _ => None,
        };

        let t1 = spawn_log_thread(BufReader::new(stdout), rflag.clone(), pkgmanager_name.clone());
        let t2 = spawn_log_thread(BufReader::new(stderr), rflag, pkgmanager_name.clone());

        t1.join().expect("stdout thread panicked");
        t2.join().expect("stderr thread panicked");

        let status = child.wait().expect("Failed to wait on package manager");
        last_exit = status.code().unwrap_or(-1);
        if !status.success() {
            error!(
                "The package manager failed with exit code: {last_exit}",
            );
        }

        // Fedora mounts per-iteration (match original behavior)
        if matches!(pkgmanager, PackageManager::Dnf | PackageManager::RpmOSTree) {
            if let Err(e) = exec::unmount_chroot_base() {
                error!("Warning: Failed to unmount chroot base: {e}");
            }
        }

        retry_counter += 1;
    }

    last_exit
}

// One helper that supports both: Arch (retry/mirror handling) and Fedora (plain log)
fn spawn_log_thread<R: BufRead + Send + 'static>(
    reader_handle: R,
    retry: Option<Arc<Mutex<bool>>>,     // Arch: Some, Fedora/Nix: None
    pkgmanager_name: Option<String>,     // Arch: Some("pacstrap"/"pacman"), otherwise None
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // don't double-wrap; we already receive a BufRead
        let mut reader = reader_handle;

        let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    // decode lossily to avoid panics on non-UTF8
                    let mut line = String::from_utf8_lossy(&buf).into_owned();
                    // trim trailing newline(s)
                    while line.ends_with('\n') || line.ends_with('\r') {
                        line.pop();
                    }

                    // always log; you can also strip control sequences if you want
                    info!("{line}");

                    // Arch-only mirror/key handling:
                    let (Some(retry_flag), Some(pm_name)) = (&retry, &pkgmanager_name) else { continue };

                    // NOTE: do NOT break the loop when we want to retry;
                    // just set the flag and KEEP reading to drain the pipe.
                    if line.contains("failed retrieving file") && line.contains("from") {
                        if let Some(mirror_name) = extract_mirror_name(&line) {
                            if let Some(mirrorlist_file) = find_mirrorlist_file(&mirror_name, pm_name) {
                                if let Err(err) = move_server_line(&mirrorlist_file, &mirror_name) {
                                    error!("Failed to move 'Server' line in {mirrorlist_file}: {err}");
                                } else {
                                    info!("Detected unstable mirror: {mirror_name}. Will retry with a new one...");
                                    *retry_flag.lock().unwrap() = true;
                                }
                            }
                        }
                    } else if (line.contains("File") && line.contains("is corrupted")) || line.contains("invalid key") {
                        let package_name = extract_package_name(&line);
                        let repository   = get_repository_name(&package_name);

                        let mirrorlist_filename = match (pm_name.as_str(), repository.as_str()) {
                            ("pacstrap", "chaotic-aur") | ("pacman", "chaotic-aur") =>
                                "/etc/pacman.d/chaotic-mirrorlist",
                            ("pacstrap", _) =>
                                "/etc/pacman.d/mirrorlist",
                            ("pacman",   _) =>
                                "/mnt/etc/pacman.d/mirrorlist",
                            _ => "",
                        }.to_string();

                        match get_first_mirror_name(&mirrorlist_filename) {
                            Ok(mirror_name) => {
                                if let Err(err) = move_server_line(&mirrorlist_filename, &mirror_name) {
                                    error!("Failed to move 'Server' line in {mirrorlist_filename}: {err}");
                                } else {
                                    info!("Detected issue on mirror: {mirror_name}. Will retry with a new one...");
                                    *retry_flag.lock().unwrap() = true;
                                }
                            }
                            Err(err) => error!("Error: {err}"),
                        }
                    }
                }
                Err(e) => {
                    // Log the read error but keep the loop going to try to drain further bytes.
                    // If you'd rather bail on persistent I/O errors, you could count consecutive errors.
                    error!("read stream error: {e}");
                    // optional: small sleep/yield here
                }
            }
        }
    })
}

// Function to extract the mirror name from the error message
fn extract_mirror_name(error_message: &str) -> Option<String> {
    // Split the error message by whitespace to get individual words
    let words: Vec<&str> = error_message.split_whitespace().collect();

    // Iterate through the words to find the word "from" and the subsequent word
    if let Some(from_index) = words.iter().position(|&word| word == "from") {
        if let Some(mirror_name) = words.get(from_index + 1) {
            return Some(mirror_name.to_string());
        }
    }

    None // Return None if no mirror name is found
}

// Function to find the mirrorlist file containing the mirror
fn find_mirrorlist_file(mirror_name: &str, pkgmanager_name: &str) -> Option<String> {
    // Define the paths to the mirrorlist files
    let mut mirrorlist_paths: [&str; 2] = ["", ""];
    if pkgmanager_name == "pacstrap" {
        mirrorlist_paths = [
            "/etc/pacman.d/mirrorlist",
            "/etc/pacman.d/chaotic-mirrorlist",
        ];
    }
    else if pkgmanager_name == "pacman" {
        mirrorlist_paths = [
            "/mnt/etc/pacman.d/mirrorlist",
            "/mnt/etc/pacman.d/chaotic-mirrorlist",
        ];
    }

    // Iterate through the mirrorlist file paths
    for &mirrorlist_path in &mirrorlist_paths {
        // Read the content of the mirrorlist file
        if let Ok(content) = fs::read_to_string(mirrorlist_path) {
            // Check if the mirror name is contained in the file content
            if content.contains(mirror_name) {
                return Some(mirrorlist_path.to_string());
            }
        }
    }

    None // Return None if the mirror name is not found in any mirrorlist file
}

// Function to move the "Server" line in the mirrorlist file
fn move_server_line(mirrorlist_path: &str, mirror_name: &str) -> io::Result<()> {
    // Read the content of the mirrorlist file
    let mut lines: Vec<String> = Vec::new();
    let file = File::open(mirrorlist_path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    // Find the index of the last line starting with "Server"
    let last_server_index = lines.iter().rposition(|line| line.trim().starts_with("Server"));

    if let Some(last_server_index) = last_server_index {
        // Find the mirror URL line
        if let Some(mirror_url_index) = lines.iter().position(|line| line.contains(mirror_name)) {
            // Extract the mirror URL line
            let mirror_url_line = lines.remove(mirror_url_index);

            // Insert the mirror URL line after the last "Server" line
            let insert_index = last_server_index;
            lines.insert(insert_index, mirror_url_line.clone());
            // Write the modified content back to the mirrorlist file
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(mirrorlist_path)?;

            for line in lines {
                writeln!(file, "{line}")?;
            }
            info!("'{mirror_url_line}' moved at the end of {mirrorlist_path}");
        }
    }

    Ok(())
}

fn get_first_mirror_name(filename: &str) -> Result<String, io::Error> {
    let file = File::open(filename)?;
    
    for line in BufReader::new(file).lines() {
        let line = line?; // Unwrap the Result to get the line directly
        if let Some(equals_index) = line.find('=') {
            let trimmed_line = line[..equals_index].trim();
            if trimmed_line == "Server" {
                let mirror_url = line[equals_index + 1..].trim();
                return Ok(mirror_url.to_string());
            }
        }
    }
    
    Err(io::Error::new(io::ErrorKind::NotFound, "Mirror not found"))
}

fn extract_package_name(line: &str) -> String {
    // Regular expression to match both patterns: 
    // - /pkg/<package-name>-version.pkg.tar (file path)
    // - error: <package-name>: invalid key found
    let re = Regex::new(r"(?:/pkg/|error:\s)([a-zA-Z0-9\-_]+)").unwrap();

    // Apply the regex to the input line
    if let Some(captures) = re.captures(line) {
        let package_with_version = captures[1].to_string();
        
        // Split the package name by "-" and remove the version part (anything after the last "-")
        if let Some((package_name, _version)) = package_with_version.rsplit_once('-') {
            return package_name.to_string();
        }

        return package_with_version; // If no version is found, return the entire capture
    }
    String::new()
}

fn get_repository_name(package_name: &str) -> String {
    // Run the `pacman -Si` command and capture its output
    let output = Command::new("pacman")
        .arg("-Si")
        .arg(package_name)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // Convert the stdout bytes to a string
            let stdout = String::from_utf8(output.stdout);
            match stdout {
                Ok(stdout) => {
                    // Find the "Repository" field in the output
                    if let Some(repository_line) = stdout.lines().find(|line| line.starts_with("Repository")) {
                        // Split the line by ':' and extract the repository name
                        let parts: Vec<&str> = repository_line.split(':').collect();
                        if parts.len() >= 2 {
                            return parts[1].trim().to_string();
                        }
                    }
                }
                Err(_) => error!("Failed to convert stdout to string"),
            }
        }
        Ok(_) => error!("Package not found"),
        Err(_) => error!("Failed to execute command"),
    }

    // Return an empty string if an error occurred
    String::new()
}
