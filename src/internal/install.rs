use crate::args::PackageManager;
use log::{error, info, warn};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn install(pkgmanager: PackageManager, pkgs: Vec<&str>) {

    // Create an Arc<Mutex<bool>> for the retry flag
    let mut retry = Arc::new(Mutex::new(true)); //Just to enter the first time in the while loop
    
    let mut retry_counter = 0; // Initialize retry counter
    while *retry.lock().unwrap() && retry_counter < 15 { // retry_counter should be the number of mirrors in mirrorlist
        retry = Arc::new(Mutex::new(false));
        let retry_clone = Arc::clone(&retry); // Clone for use in the thread. I need to do this because normally I cannot define a variable above and use it inside a threadzz
        //log::info!("[ DEBUG ] Beginning retry {}", *retry.lock().unwrap());
        let mut pkgmanager_cmd = Command::new("true")
            .spawn()
            .expect("Failed to initiialize by 'true'"); // Note that the Command type below will spawn child process, so the return type is Child, not Command. It means we need to initialize a Child type element, and we can do by .spawn().expect() over the Command type. 'true' in bash is like a NOP command
        let mut pkgmanager_name = String::new();
        match pkgmanager {
            PackageManager::Pacman => {
                pkgmanager_cmd = Command::new("arch-chroot")
                    .arg("/mnt")
                    .arg("pacman")
                    .arg("-Syyu")
                    .arg("--needed")
                    .arg("--noconfirm")
                    .args(&pkgs)
                    .stdout(Stdio::piped()) // Capture stdout
                    .stderr(Stdio::piped()) // Capture stderr
                    .spawn()
                    .expect("Failed to start pacman");
                pkgmanager_name = String::from("pacman");
            },
            PackageManager::Pacstrap => {
                pkgmanager_cmd = Command::new("pacstrap")
                    .arg("/mnt")
                    .args(&pkgs)
                    .stdout(Stdio::piped()) // Capture stdout
                    .stderr(Stdio::piped()) // Capture stderr
                    .spawn()
                    .expect("Failed to start pacstrap");
                pkgmanager_name = String::from("pacstrap");
            },
            PackageManager::None => log::debug!("No package manager selected"),
        };

        let stdout_handle = pkgmanager_cmd.stdout.take().unwrap();
        let stderr_handle = pkgmanager_cmd.stderr.take().unwrap();

        let stdout_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout_handle);
            for line in reader.lines() {
                let line = line.expect("Failed to read stdout");
                info!("{}", line);
            }
        });

        let exit_status = pkgmanager_cmd.wait().expect("Failed to wait for the package manager");

        let stderr_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr_handle);
            for line in reader.lines() {
                if *retry_clone.lock().unwrap() {
                    break; // Exit the for loop early if *retry is true. It means we updated the mirrorlist, we can proceed to retry the install command
                }
                let line = line.expect("Failed to read stderr");
                let exit_code = exit_status.code().unwrap_or(-1);
                if exit_code == 0 {
                    warn!(
                        "{} warn (exit code {}): {}",
                        pkgmanager_name,
                        exit_code,
                        line
                    );
                }
                else {
                    error!(
                        "{} err (exit code {}): {}",
                        pkgmanager_name,
                        exit_code,
                        line
                    );
                }

                // Check if the error message contains "failed retrieving file" and "mirror"
                if line.contains("failed retrieving file") && line.contains("from") {
                    // Extract the mirror name from the error message
                    if let Some(mirror_name) = extract_mirror_name(&line) {
                        // Check if the mirror is in one of the mirrorlist files
                        if let Some(mirrorlist_file) = find_mirrorlist_file(&mirror_name, &pkgmanager_name) {
                            // Move the "Server" line within the mirrorlist file
                            if let Err(err) = move_server_line(&mirrorlist_file, &mirror_name) {
                                error!(
                                    "Failed to move 'Server' line in {}: {}",
                                    mirrorlist_file,
                                    err
                                );
                            } else {
                                // Update the retry flag within the Mutex
                                log::info!("Detected unstable mirror: {}. Retrying by a new one...", mirror_name);
                                let mut retry = retry_clone.lock().unwrap();
                                *retry = true;
                                //log::info!("[ DEBUG ] Unstable mirror retry {}", *retry);
                            }
                        }
                    }
                }
                else if line.contains("signature from") && line.contains("is invalid") {
                    let package_name = extract_package_name(&line);
                    let repository = get_repository_name(&package_name);
                    println!("Package {} found in repository: {}", package_name, repository);
                    let mut mirrorlist_filename = String::new();
                    if pkgmanager_name == "pacstrap" {
                        if repository == "core" || repository == "extra" || repository == "community" || repository == "multilib" {
                            mirrorlist_filename = String::from("/etc/pacman.d/mirrorlist");
                        }
                        if repository == "blackarch" {
                            mirrorlist_filename = String::from("/etc/pacman.d/blackarch-mirrorlist");
                        }
                        if repository == "chaotic-aur" {
                            mirrorlist_filename = String::from("/etc/pacman.d/chaotic-mirrorlist");
                        }
                    }
                    else if pkgmanager_name == "pacman" {
                        if repository == "core" || repository == "extra" || repository == "community" || repository == "multilib" {
                            mirrorlist_filename = String::from("/mnt/etc/pacman.d/mirrorlist");
                        }
                        if repository == "blackarch" {
                            mirrorlist_filename = String::from("/mnt/etc/pacman.d/blackarch-mirrorlist");
                        }
                        if repository == "chaotic-aur" {
                            mirrorlist_filename = String::from("/mnt/etc/pacman.d/chaotic-mirrorlist");
                        }
                    }
                    
                    match get_first_mirror_name(&mirrorlist_filename) {
                        Ok(mirror_name) => {
                            println!("Mirror Name: {}", mirror_name);
                            if let Err(err) = move_server_line(&mirrorlist_filename, &mirror_name) {
                                error!(
                                    "Failed to move 'Server' line in {}: {}",
                                    mirrorlist_filename,
                                    err
                                );
                            } else {
                                // Update the retry flag within the Mutex
                                log::info!("Detected invalid signature key in mirror: {}. Retrying by a new one...", mirror_name);
                                let mut retry = retry_clone.lock().unwrap();
                                *retry = true;
                                //log::info!("[ DEBUG ] Invalid signature key in mirror retry {}", *retry);
                            }
                        }
                        Err(err) => eprintln!("Error: {}", err),
                    }
                }
            }
        });

        // Wait for the stdout and stderr threads to finish
        stdout_thread.join().expect("stdout thread panicked");
        stderr_thread.join().expect("stderr thread panicked");

        if !exit_status.success() {
            // Handle the error here, e.g., by logging it
            error!("The package manager failed with exit code: {}", exit_status.code().unwrap_or(-1));
        }

        // Increment the retry counter
        retry_counter += 1;

        //log::info!("[ DEBUG ] End retry {}", *retry.lock().unwrap());
    }
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
    let mut mirrorlist_paths: [&str; 3] = ["", "", ""];
    if pkgmanager_name == "pacstrap" {
        mirrorlist_paths = [
            "/etc/pacman.d/mirrorlist",
            "/etc/pacman.d/chaotic-mirrorlist",
            "/etc/pacman.d/blackarch-mirrorlist",
        ];
    }
    else if pkgmanager_name == "pacman" {
        mirrorlist_paths = [
            "/mnt/etc/pacman.d/mirrorlist",
            "/mnt/etc/pacman.d/chaotic-mirrorlist",
            "/mnt/etc/pacman.d/blackarch-mirrorlist",
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
                writeln!(file, "{}", line)?;
            }
            log::info!("'{}' moved at the end of {}", mirror_url_line, mirrorlist_path);
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

fn extract_package_name(input: &str) -> String {
    let error_prefix = "error:";
    let colon = ':';

    if let Some(error_idx) = input.find(error_prefix) {
        let remaining_text = &input[error_idx + error_prefix.len()..];
        if let Some(colon_idx) = remaining_text.find(colon) {
            let package_name = &remaining_text[..colon_idx].trim();
            return package_name.to_string();
        }
    }
    String::new() // Return an empty string if package name is not found
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
                Err(_) => eprintln!("Failed to convert stdout to string"),
            }
        }
        Ok(_) => eprintln!("Package not found"),
        Err(_) => eprintln!("Failed to execute command"),
    }

    // Return an empty string if an error occurred
    String::new()
}