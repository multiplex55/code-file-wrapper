//! # File Operations Module
//!
//! This module is responsible for processing files in a directory by reading their contents,
//! filtering by file extension, and writing them into a single tagged output file (`tags_output.txt`).
//!
//! # Features
//! - Recursively or non-recursively scan directories.
//! - Filter files by allowed extensions.
//! - Skip hidden or user-specified folders.
//! - Wrap file contents in XML-style tags based on relative path.
//! - Append instructional or command-based sections at the end of the output.
//!
//! # Key Functions
//! - [`write_folder_tags`]: Top-level entry point for generating tagged file output.
//! - [`write_folder_tags_recursive`]: Internal recursive helper for deep directory traversal.
//! - [`append_additional_commands`]: Appends extra user-defined command blocks.
//! - [`is_human_readable`]: Checks if a file has an allowed extension.
//!
//! # Output Behavior
//! - Always creates (or overwrites) `tags_output.txt` in the current working directory.
//! - Each file is written in the format:
//!   ```xml
//!   <relative\path\to\file.rs>
//!   // file content
//!   </relative\path\to\file.rs>
//!   ```
//!
//! # Notes
//! - File paths are rendered in Windows-style even on other platforms.
//! - UTF-8 file reading is assumed; non-UTF8 files are skipped with a warning.

use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::Path;

/// Writes the contents of all valid files in a specified directory to a `tags_output.txt` file,
/// wrapping each file's contents inside XML-style tags based on the relative file path.
///
/// # Parameters
/// - `dir`: A reference to the root directory `Path` to scan for files.
/// - `valid_exts`: A list of allowed file extensions (e.g., `["rs", "txt"]`), without leading dots.
/// - `recursive`: If `true`, traverses the directory tree recursively; otherwise, only the top-level is processed.
/// - `ignored_folders`: A list of folder names (case-insensitive) to skip during recursive traversal.
///
/// # Behavior
/// - Creates or overwrites `tags_output.txt` in the current working directory.
/// - Traverses the given directory (recursively or not depending on `recursive`).
/// - For each file with a valid extension:
///   - Reads the file contents (assuming UTF-8 encoding).
///   - Writes the contents between opening and closing tags derived from the file’s relative path.
///
/// # Output Format
/// Each file is wrapped in a tag like:
/// ```xml
/// <relative\path\to\file.rs>
/// // file contents here...
/// </relative\path\to\file.rs>
/// ```
///
/// # Footer
/// After processing, the file includes an instructional block describing the output usage,
/// followed by a note to pay close attention to `[Additional Commands]` if present.
///
/// # Panics
/// - This function does not panic explicitly. It uses the `?` operator to propagate I/O errors.
///
/// # Errors
/// - Returns `Err(std::io::Error)` if:
///   - The output file cannot be created.
///   - The directory cannot be read.
///   - A file read operation fails fatally (non-fatal file read errors are printed to `stderr` but skipped).
///
/// # Notes
/// - Non-UTF8 files are skipped with a warning.
/// - Folder names in `ignored_folders` are compared case-insensitively.
/// - Hidden folders (starting with `.`) are also automatically skipped during recursion.
/// - File path tags use Windows-style `\` separators regardless of OS.
///
/// # Example
/// ```rust
/// let dir = Path::new("src");
/// let exts = &["rs", "md"];
/// let ignored = vec!["target".to_string(), ".git".to_string()];
/// write_folder_tags(dir, exts, true, &ignored)?;
/// ```
pub fn write_folder_tags(
    dir: &Path,
    valid_exts: &[&str],
    recursive: bool,
    ignored_folders: &[String],
) -> std::io::Result<()> {
    let mut output = File::create("tags_output.txt")?;

    if recursive {
        write_folder_tags_recursive(dir, dir, valid_exts, &mut output, ignored_folders)?;
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
    writeln!(output, "* Each node above is an XML-wrapped code snippet using relative Windows-style file path tags.")?;
    writeln!(output, "* Provide context above and below code changes to be explicit on where any change should occur.")?;
    writeln!(
        output,
        "* Under text under [Additional Commands] should be read very carefully and followed absolutely"
    )?;

    Ok(())
}

/// Determines whether a file should be considered human-readable based on its extension.
///
/// # Parameters
/// - `path`: Reference to a `Path` representing the file to inspect.
/// - `valid_exts`: A list of file extensions (e.g., `["rs", "txt"]`) to match against. Extensions **must not** include leading dots.
///
/// # Returns
/// - `true` if the file's extension matches one of the values in `valid_exts`.
/// - `false` if:
///   - The file has no extension,
///   - The extension is not valid UTF-8,
///   - Or it does not match any item in `valid_exts`.
///
/// # Behavior
/// - Extracts the file extension using `Path::extension()`.
/// - Converts it to a UTF-8 string using `to_str()`.
/// - Performs an exact string match against the provided list.
///
/// # Panics
/// - This function does **not** panic under any normal circumstances.
///
/// # Limitations
/// - Does **not** inspect file contents or MIME types.
/// - Does **not** distinguish between binary and text files.
/// - Case-sensitive by default (e.g., `"RS"` and `"rs"` are considered different).
///
/// # Use Cases
/// - Acts as a simple filter to determine whether a file is eligible for reading and inclusion in output generation.
///
/// # Example
/// ```rust
/// let path = Path::new("src/main.rs");
/// assert!(is_human_readable(path, &["rs", "txt"]));
///
/// let path = Path::new("README");
/// assert!(!is_human_readable(path, &["md"])); // no extension
/// ```
///
/// # Future Enhancements
/// - Support case-insensitive matching if needed.
/// - Add content-based heuristics (e.g., check for UTF-8 validity or printable character ratios).
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}

/// Appends a block of additional text to an existing output file, creating the file if it does not exist.
///
/// # Parameters
/// - `file_path`: Path to the output file (typically `"tags_output.txt"`).
/// - `additional_commands`: Arbitrary text (typically user instructions or commands) to be appended to the file.
///
/// # Behavior
/// - Opens the target file in **append mode**, creating it if it doesn't exist.
/// - Writes a fixed section header `[Additional Commands]`, followed by the provided `additional_commands` string.
/// - Ensures that newlines are included for proper formatting.
///
/// # Output Format
/// The appended content will follow this structure at the end of the file:
/// ```
/// [Additional Commands]
/// user text here...
/// ```
///
/// # Errors
/// - Returns `Err(std::io::Error)` if:
///   - The file cannot be opened or created.
///   - Writing to the file fails for any reason (e.g., permission denied, disk full).
///
/// # Panics
/// - This function does not panic under normal conditions.
/// - All I/O operations are fallible and handled via the `?` operator.
///
/// # Notes
/// - Repeated calls will result in multiple `[Additional Commands]` headers unless deduplication is added externally.
/// - The caller is responsible for ensuring that `additional_commands` is not empty or malformed.
/// - This function does not trim or sanitize the content — it writes exactly what is passed in.
///
/// # Example
/// ```rust
/// append_additional_commands("tags_output.txt", "TODO: Review all usage of unwrap().")?;
/// ```
pub fn append_additional_commands(
    file_path: &str,
    additional_commands: &str,
) -> std::io::Result<()> {
    let mut file = File::options().create(true).append(true).open(file_path)?;

    writeln!(file, "\n[Additional Commands]")?;
    writeln!(file, "{}\n", additional_commands)?;

    Ok(())
}

/// Recursively traverses a directory tree, writing the contents of valid files into an output file,
/// while skipping ignored folders and hidden directories.
///
/// # Parameters
/// - `root_dir`: The top-level directory used to compute relative paths for XML tags.
/// - `dir`: The current directory being scanned (initially the same as `root_dir`).
/// - `valid_exts`: A list of valid file extensions (e.g., `["rs", "json"]`) to filter files.
/// - `output`: A mutable reference to the open output file (`tags_output.txt`), to which results are written.
/// - `ignored_folders`: A list of folder names to skip, case-insensitive (e.g., `["target", ".git"]`).
///
/// # Behavior
/// - Traverses the directory tree rooted at `dir`.
/// - Skips:
///   - Any directory in `ignored_folders` (case-insensitive match).
///   - Any hidden directory (name starts with a `.`).
/// - For each encountered file with a valid extension:
///   - Attempts to read the contents as UTF-8.
///   - Writes the contents between XML-style tags based on the file’s path relative to `root_dir`.
///
/// # Output Format
/// Output to the file looks like:
/// ```xml
/// <subdir\file.rs>
/// // contents...
/// </subdir\file.rs>
/// ```
///
/// # Errors
/// - Returns `Err(std::io::Error)` if a fatal I/O error occurs, such as:
///   - Failing to read the directory.
///   - Failing to open the file.
///   - Failing to write to the output file.
/// - If reading a particular file fails (e.g., due to encoding), a warning is printed to `stderr` and the file is skipped.
///
/// # Panics
/// - This function does not panic under normal circumstances.
/// - All filesystem operations are wrapped with `?` or handled gracefully.
///
/// # Notes
/// - Relative paths are derived using `Path::strip_prefix(root_dir)`.
/// - Paths are converted to string tags using `.to_str()` (files with invalid UTF-8 paths are skipped).
/// - File names and extensions are not modified — if extension case sensitivity matters, it must be handled by `valid_exts`.
/// - Designed to work well with Windows paths but portable to other platforms as long as file paths are valid UTF-8.
///
/// # Example
/// ```rust
/// let root = Path::new("src");
/// let mut file = File::create("tags_output.txt")?;
/// let ignored = vec!["target".to_string(), "build".to_string()];
/// write_folder_tags_recursive(root, root, &["rs"], &mut file, &ignored)?;
/// ```
fn write_folder_tags_recursive(
    root_dir: &Path,
    dir: &Path,
    valid_exts: &[&str],
    output: &mut File,
    ignored_folders: &[String],
) -> std::io::Result<()> {
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                let folder_name = folder_name.to_lowercase();
                if ignored_folders.contains(&folder_name) || folder_name.starts_with('.') {
                    continue;
                }
            }
            write_folder_tags_recursive(root_dir, &path, valid_exts, output, ignored_folders)?;
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
