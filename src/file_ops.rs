//! # File Operations Module
//!
//! This module manages reading file contents, filtering based on extensions,
//! and writing them into `tags_output.txt` in an XML-like format.

use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::Path;

/// Writes the contents of all valid files in a directory to `tags_output.txt` with XML-like tags.
///
/// # Parameters
/// - `dir`: Path to the directory containing files.
/// - `valid_exts`: List of valid file extensions.
///
/// # Returns
/// - `Ok(())` if successful, otherwise an `Err`.
pub fn write_folder_tags(dir: &Path, valid_exts: &[&str]) -> std::io::Result<()> {
    let mut output = File::create("tags_output.txt")?;

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_human_readable(&path, valid_exts) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let mut contents = String::new();
                File::open(&path)?.read_to_string(&mut contents)?;
                writeln!(output, "<{}>", name)?;
                writeln!(output, "{}", contents)?;
                writeln!(output, "</{}>\n", name)?;
            }
        }
    }

    // Append instructional text
    writeln!(output, "* The above is the current state of my project.")?;
    writeln!(output, "* Provide context above and below code changes.")?;
    writeln!(
        output,
        "* Be extremely explicit with where to make changes."
    )?;

    Ok(())
}

/// Checks if a file has a valid human-readable extension.
///
/// # Parameters
/// - `path`: Path to the file.
/// - `valid_exts`: List of valid extensions.
///
/// # Returns
/// - `true` if the file is readable, `false` otherwise.
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}

/// Appends additional commands or text to `tags_output.txt`.
///
/// # Parameters
/// - `file_path`: Path to the output file.
/// - `additional_commands`: The text to append.
///
/// # Returns
/// - `Ok(())` if successful, otherwise an `Err`.
pub fn append_additional_commands(
    file_path: &str,
    additional_commands: &str,
) -> std::io::Result<()> {
    let mut file = File::options().create(true).append(true).open(file_path)?;

    writeln!(file, "\n[Additional Commands]")?;
    writeln!(file, "{}\n", additional_commands)?;

    Ok(())
}
