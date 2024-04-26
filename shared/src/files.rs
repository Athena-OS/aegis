use crate::log::{info};
use crate::strings::crash;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Error, ErrorKind};
use std::path::Path;
use regex::Regex;

pub fn create_file(path: &str) {
    let returncode = File::create(path);
    match returncode {
        Ok(_) => {
            info!("Create {}", path);
        }
        Err(e) => {
            crash(format!("Create {}: Failed with error {}", path, e), 1);
        }
    }
}

pub fn copy_file(path: &str, destpath: &str) {
    let return_code = std::fs::copy(path, destpath);
    match return_code {
        Ok(_) => {
            info!("Copy {} to {}", path, destpath);
        }
        Err(e) => {
            crash(
                format!("Copy {} to {}: Failed with error {}", path, destpath, e),
                1,
            );
        }
    }
}

// Copy recusrively all files from the given source directory to the specified target directory
pub fn copy_all_files(source_dir: &str, target_dir: &str) {
    // Convert &str paths to Path
    let source_path = Path::new(source_dir);
    let target_path = Path::new(target_dir);

    // Create the target directory if it doesn't exist
    if let Err(err) = fs::create_dir_all(target_path) {
        println!("Error creating target directory: {}", err);
        return;
    }

    // Iterate over the entries in the source directory
    for entry in fs::read_dir(source_path).unwrap() {
        let entry = entry.unwrap();
        let source_entry_path = entry.path();

        // Construct the target path by appending the file/directory name to the target directory
        let target_entry_path = target_path.join(entry.file_name());

        // Check if the entry is a file or a directory
        if entry.file_type().unwrap().is_file() {
            // If it's a file, copy it to the target directory
            if let Err(err) = fs::copy(&source_entry_path, &target_entry_path) {
                println!("Error copying file: {}", err);
            } else {
                println!("File copied successfully: {:?}", target_entry_path);
            }
        } else if entry.file_type().unwrap().is_dir() {
            // If it's a directory, recursively copy its contents
            copy_all_files(
                &source_entry_path.to_string_lossy(),
                &target_entry_path.to_string_lossy(),
            );
        }
    }
}

pub fn rename_file(path: &str, destpath: &str) {
    let return_code = std::fs::rename(path, destpath);
    match return_code {
        Ok(_) => {
            info!("Rename {} to {}", path, destpath);
        }
        Err(e) => {
            crash(
                format!("Rename {} to {}: Failed with error {}", path, destpath, e),
                1,
            );
        }
    }
}

pub fn remove_file(path: &str) {
    let returncode = std::fs::remove_file(path);
    match returncode {
        Ok(_) => {
            info!("Remove {}", path);
        }
        Err(e) => {
            crash(format!("Remove {}: Failed with error {}", path, e), 1);
        }
    }
}

pub fn append_file(path: &str, content: &str) -> std::io::Result<()> {
    info!("Append '{}' to file {}", content.trim_end(), path);
    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(format!("{content}\n").as_bytes())?;
    Ok(())
}

pub fn sed_file(path: &str, find: &str, replace: &str) -> io::Result<()> {
    info!("Sed '{}' to '{}' in file {}", find, replace, path);
    let contents = fs::read_to_string(path)?;
    let regex = Regex::new(find).map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
    let new_contents = regex.replace_all(&contents, replace);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_contents.as_bytes())?;
    Ok(())
}

pub fn replace_line_in_file(path: &str, search: &str, replacement: &str) -> io::Result<()> {
    // Set as debug! to not log hashes during the OS install
    //debug!("Replace '{}' with '{}' in file {}", search, replacement, path);
    let contents = fs::read_to_string(path)?;
    let mut new_contents = String::new();

    for line in contents.lines() {
        if line.contains(search) {
            new_contents.push_str(replacement);
        } else {
            new_contents.push_str(line);
        }
        new_contents.push('\n');
    }

    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_contents.as_bytes())?;
    Ok(())
}

pub fn create_directory(path: &str) -> std::io::Result<()> { // Create all missing dirs in the specified path
    std::fs::create_dir_all(path)
}
