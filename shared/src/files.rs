use crate::strings::crash;
use glob::glob;
use log::{info};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Error, ErrorKind};
use regex::Regex;

pub fn create_file(path: &str) {
    let returncode = File::create(path);
    match returncode {
        Ok(_) => {
            info!("Create {path}");
        }
        Err(e) => {
            crash(format!("Create {path}: Failed with error {e}"), 1);
        }
    }
}

pub fn copy_file(path: &str, destpath: &str) {
    let return_code = std::fs::copy(path, destpath);
    match return_code {
        Ok(_) => {
            info!("Copy {path} to {destpath}");
        }
        Err(e) => {
            crash(
                format!("Copy {path} to {destpath}: Failed with error {e}"),
                1,
            );
        }
    }
}

pub fn copy_multiple_files(pattern: &str, dest_dir: &str) {
    if let Err(e) = create_directory(dest_dir) {
        crash(format!("Failed to create directory {dest_dir}: {e}"), 1);
    }

    match glob(pattern) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(path) => {
                        if path.is_file() {
                            let file_name = path.file_name().unwrap().to_string_lossy();
                            let dest_path = format!("{dest_dir}/{file_name}");
                            copy_file(path.to_str().unwrap(), &dest_path);
                        }
                    }
                    Err(e) => {
                        crash(format!("Error processing pattern {pattern}: {e}"), 1);
                    }
                }
            }
        }
        Err(e) => {
            crash(format!("Invalid glob pattern {pattern}: {e}"), 1);
        }
    }
}

pub fn rename_file(path: &str, destpath: &str) {
    let return_code = std::fs::rename(path, destpath);
    match return_code {
        Ok(_) => {
            info!("Rename {path} to {destpath}");
        }
        Err(e) => {
            crash(
                format!("Rename {path} to {destpath}: Failed with error {e}"),
                1,
            );
        }
    }
}

pub fn remove_file(path: &str) {
    let returncode = std::fs::remove_file(path);
    match returncode {
        Ok(_) => {
            info!("Remove {path}");
        }
        Err(e) => {
            crash(format!("Remove {path}: Failed with error {e}"), 1);
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
    info!("Sed '{find}' to '{replace}' in file {path}");
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
