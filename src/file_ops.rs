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
/// - `dir`: A reference to a `Path` representing the directory to scan for files.
/// - `valid_exts`: A slice of string slices (`&[&str]`) specifying the valid file extensions that should be processed.
///
/// # Returns
/// - `Ok(())`: If all file processing and writing operations complete successfully.
/// - `Err(std::io::Error)`: If any I/O operation fails (e.g., file read/write error, directory access failure).
///
/// # Behavior
/// - Iterates over all files in the given directory.
/// - Filters files based on their extensions, only processing those listed in `valid_exts`.
/// - Reads the contents of each valid file and writes them into `tags_output.txt`, wrapping the content within XML-like tags:
///   ```xml
///   <filename.rs>
///   (file contents)
///   </filename.rs>
///   ```
/// - Appends an instructional footer to guide further modifications.
///
/// # Error Handling
/// - If the directory does not exist or cannot be read, an error is returned.
/// - If `tags_output.txt` cannot be created or written to, an error is returned.
/// - If a file cannot be opened or read, a warning is printed to `stderr`, and the function continues processing other files.
///
/// # Panics
/// - This function does not explicitly panic but relies on the `?` operator for error propagation.
///
/// # Example Usage
/// ```rust
/// use std::path::Path;
///
/// let dir = Path::new("src");
/// let valid_exts = &["rs", "txt", "json"];
/// if let Err(e) = write_folder_tags(dir, valid_exts) {
///     eprintln!("Error writing folder tags: {}", e);
/// }
/// ```
///
/// # Notes
/// - The function overwrites `tags_output.txt` on each execution.
/// - If a file does not contain valid UTF-8 content, reading may fail.
/// - The appended instructions in the output file guide users on how to structure modifications.
pub fn write_folder_tags(dir: &Path, valid_exts: &[&str], recursive: bool) -> std::io::Result<()> {
    let mut output = File::create("tags_output.txt")?;

    if recursive {
        write_folder_tags_recursive(dir, dir, valid_exts, &mut output)?;
    } else {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_human_readable(&path, valid_exts) {
                if let Ok(relative_path) = path.strip_prefix(dir) {
                    if let Some(rel_str) = relative_path.to_str() {
                        let mut contents = String::new();
                        File::open(&path)?.read_to_string(&mut contents)?;
                        writeln!(output, "<{}>", rel_str)?;
                        writeln!(output, "{}", contents)?;
                        writeln!(output, "</{}>\n", rel_str)?;
                    }
                }
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

/// Determines whether a given file has a human-readable extension.
///
/// # Parameters
/// - `path`: A reference to a `Path`, representing the file to check.
/// - `valid_exts`: A slice of string slices (`&[&str]`) containing the allowed file extensions.
///
/// # Returns
/// - `true` if the file has an extension that exists in `valid_exts`.
/// - `false` if the file has no extension or if its extension is not in `valid_exts`.
///
/// # Behavior
/// - Extracts the file extension from the provided `Path`.
/// - Converts the extension to a string and checks for its presence in `valid_exts`.
///
/// # Error Handling
/// - This function does not return errors.
/// - If a file has no extension, it is treated as unreadable (`false`).
///
/// # Panics
/// - This function does not panic under normal circumstances.
///
/// # Example Usage
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
///
/// # Notes
/// - The function does not check for actual file readability (e.g., permissions).
/// - Only considers file extensions, meaning that a file with a valid extension but binary content may still be considered "readable."
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}

/// Appends additional commands or text to an existing file.
///
/// # Parameters
/// - `file_path`: A string slice (`&str`) representing the path to the file where the commands should be appended.
/// - `additional_commands`: A string slice (`&str`) containing the text to be appended.
///
/// # Returns
/// - `Ok(())` if the text is successfully appended.
/// - `Err(std::io::Error)` if an error occurs during file operations.
///
/// # Behavior
/// - Opens (or creates if it does not exist) the file specified by `file_path` in append mode.
/// - Writes a header section `"[Additional Commands]"` followed by the content of `additional_commands`.
/// - Ensures each new entry is separated by a newline.
///
/// # Error Handling
/// - If the file cannot be opened or created, an `Err(std::io::Error)` is returned.
/// - If writing to the file fails, an `Err(std::io::Error)` is returned.
///
/// # Panics
/// - This function does not panic under normal conditions.
///
/// # Example Usage
/// ```rust
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let file_path = "tags_output.txt";
///     let commands = "TODO: Implement additional logic.";
///
///     append_additional_commands(file_path, commands)?;
///     println!("Commands appended successfully.");
///
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - If `file_path` does not exist, it will be created.
/// - If `additional_commands` is an empty string, only the header section will be written.
/// - Calling this function multiple times appends new entries rather than overwriting existing content.
pub fn append_additional_commands(
    file_path: &str,
    additional_commands: &str,
) -> std::io::Result<()> {
    let mut file = File::options().create(true).append(true).open(file_path)?;

    writeln!(file, "\n[Additional Commands]")?;
    writeln!(file, "{}\n", additional_commands)?;

    Ok(())
}

fn write_folder_tags_recursive(
    root_dir: &Path,
    dir: &Path,
    valid_exts: &[&str],
    output: &mut File,
) -> std::io::Result<()> {
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            write_folder_tags_recursive(root_dir, &path, valid_exts, output)?;
        } else if path.is_file() && is_human_readable(&path, valid_exts) {
            if let Ok(relative_path) = path.strip_prefix(root_dir) {
                if let Some(rel_str) = relative_path.to_str() {
                    let mut contents = String::new();
                    File::open(&path)?.read_to_string(&mut contents)?;
                    writeln!(output, "<{}>", rel_str)?;
                    writeln!(output, "{}", contents)?;
                    writeln!(output, "</{}>\n", rel_str)?;
                }
            }
        }
    }
    Ok(())
}
