//! # File Operations Module
//!
//! This file contains the functions responsible for creating the output
//! file (`tags_output.txt`) and filtering out files that do not have
//! human-readable extensions (like `.exe` or `.pdb`).

use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Writes each file in the folder into an output file with tags.
/// Only includes files with extensions typically opened in text editors.
pub fn write_folder_tags(dir: &Path) -> std::io::Result<()> {
    let valid_exts = ["ini", "txt", "rs", "cs", "json", "xml"];
    let mut output = File::create("tags_output.txt")?;
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_human_readable(&path, &valid_exts) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let mut contents = String::new();
                File::open(&path)?.read_to_string(&mut contents)?;
                writeln!(output, "<{}>", name)?;
                writeln!(output, "{}", contents)?;
                writeln!(output, "</{}>\n", name)?;
            }
        }
    }
    Ok(())
}

/// Returns `true` if the file has an extension matching one in `valid_exts`.
///
/// # Parameters
///
/// * `path` - The path to the file.
/// * `valid_exts` - A list of valid file extensions.
///
/// # Returns
///
/// * `bool` indicating whether the file is considered human-readable.
fn is_human_readable(path: &PathBuf, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}
