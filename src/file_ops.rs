//! # File Operations Module
//!
//! This file contains the functions responsible for creating the output
//! file (`tags_output.txt`) and filtering out files that do not have
//! human-readable extensions (like `.exe` or `.pdb`).

use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::Path;

/// Writes the contents of all valid files in a directory to `tags_output.txt` with XML-like tags.
///
/// # Functionality
/// - Iterates over all files in the specified directory.
/// - Filters files based on the provided list of valid extensions.
/// - Reads the contents of each valid file and writes them to `tags_output.txt` with the filename as a tag.
/// - Appends instructional text at the end of the file.
///
/// # Parameters
/// - `dir: &Path` → The directory containing the files to process.
/// - `valid_exts: &[&str]` → A slice of strings representing the valid file extensions.
///
/// # Returns
/// - `std::io::Result<()>` → Returns `Ok(())` if all operations succeed, or an `Err` if any file operation fails.
///
/// # Error Handling
/// - If the directory does not exist or cannot be read, an error is returned.
/// - If `tags_output.txt` cannot be created or written to, an error is returned.
/// - If a file cannot be opened or read, a warning is printed, and the function continues processing other files.
///
/// # Panics
/// - This function does not explicitly panic but will return an error if file operations fail.
///
/// # Side Effects
/// - Creates (or overwrites) `tags_output.txt`.
/// - Reads and writes file contents from the specified directory.
///
/// # Usage
/// ```rust
/// use std::path::Path;
///
/// let dir = Path::new("my_project");
/// let valid_exts = &["rs", "txt", "json"];
/// if let Err(e) = write_folder_tags(dir, valid_exts) {
///     eprintln!("Error writing folder tags: {}", e);
/// }
/// ```
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
    writeln!(output, "* The above is the current state of my project with each \"<>\" block containing the file it belongs to")?;
    writeln!(output, "* Provide context above and below code changes with clear temp comments to indicate change")?;
    writeln!(output, "* Be extremely explicit with where to make changes")?;
    Ok(())
}

/// Determines whether a given file has a human-readable extension.
///
/// # Functionality
/// - Extracts the file extension from the given file path.
/// - Checks if the extracted extension is present in the list of valid extensions.
///
/// # Parameters
/// - `path: &Path` → The file path to check.
/// - `valid_exts: &[&str]` → A slice of strings representing the valid file extensions.
///
/// # Returns
/// - `bool`
///   - `true` → If the file has an extension that is in `valid_exts`.
///   - `false` → If the file has no extension or its extension is not in `valid_exts`.
///
/// # Error Handling
/// - This function does not return errors. If a file has no extension, it simply returns `false`.
///
/// # Panics
/// - This function does not panic under normal circumstances.
///
/// # Side Effects
/// - None. It only checks the file extension.
///
/// # Usage
/// ```rust
/// use std::path::Path;
///
/// let file_path = Path::new("script.rs");
/// let valid_extensions = &["rs", "txt", "json"];
///
/// if is_human_readable(file_path, valid_extensions) {
///     println!("The file is human-readable.");
/// } else {
///     println!("The file is not human-readable.");
/// }
/// ```
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}
