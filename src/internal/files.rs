use crate::internal::*;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Error, ErrorKind};
use regex::Regex;

pub fn create_file(path: &str) {
    let returncode = File::create(path);
    match returncode {
        Ok(_) => {
            log::info!("Create {}", path);
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
            log::info!("Copy {} to {}", path, destpath);
        }
        Err(e) => {
            crash(
                format!("Copy {} to {}: Failed with error {}", path, destpath, e),
                1,
            );
        }
    }
}

pub fn rename_file(path: &str, destpath: &str) {
    let return_code = std::fs::rename(path, destpath);
    match return_code {
        Ok(_) => {
            log::info!("Rename {} to {}", path, destpath);
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
            log::info!("Remove {}", path);
        }
        Err(e) => {
            crash(format!("Remove {}: Failed with error {}", path, e), 1);
        }
    }
}

pub fn append_file(path: &str, content: &str) -> std::io::Result<()> {
    log::info!("Append '{}' to file {}", content.trim_end(), path);
    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(format!("\n{content}\n").as_bytes())?;
    Ok(())
}

pub fn sed_file(path: &str, find: &str, replace: &str) -> io::Result<()> {
    log::info!("Sed '{}' to '{}' in file {}", find, replace, path);
    let contents = fs::read_to_string(path)?;
    let regex = Regex::new(find).map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
    let new_contents = regex.replace_all(&contents, replace);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_contents.as_bytes())?;
    Ok(())
}

pub fn create_directory(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
