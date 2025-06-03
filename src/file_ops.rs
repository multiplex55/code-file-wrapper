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

/// Writes the contents of selected files in a directory into a tagged output file (`tags_output.txt`).
///
/// # Purpose
/// Collects all files matching specific extensions from a given directory, optionally recursively,
/// and writes each file’s contents into `tags_output.txt`, wrapped in an XML-style tag that corresponds
/// to its relative path.
///
/// # Parameters
/// - `dir`: Root directory to scan (`&Path`).
/// - `valid_exts`: List of allowed file extensions (e.g., `["rs", "md"]`) — case-sensitive and without dots.
/// - `recursive`: If `true`, traverses subdirectories. If `false`, only scans the top-level directory.
/// - `ignored_folders`: List of folder names (case-insensitive) to skip during recursive traversal (e.g., `["target", ".git"]`).
///
/// # Output Format
/// Each file is wrapped in tags representing its relative path:
/// ```xml
/// <src\main.rs>
/// // file contents here...
/// </src\main.rs>
/// ```
///
/// # Footer
/// After all files are written, an instructional block is appended to the file that:
/// - Explains the output format and purpose.
/// - Instructs users to carefully review appended command blocks.
/// - Prepares the result for downstream AI-assisted editing or transformation.
///
/// # Behavior
/// - All matching files are assumed to be UTF-8.
/// - Files with unreadable contents (non-UTF8 or access errors) are skipped with a warning to `stderr`.
/// - Uses Windows-style `\` in tag paths, even on other operating systems.
///
/// # Errors
/// Returns `Err(std::io::Error)` if:
/// - The directory or any file fails to open/read.
/// - The output file (`tags_output.txt`) cannot be created or written.
///
/// # Panics
/// This function does **not** panic.
/// All filesystem I/O is handled using `?` or skipped safely.
///
/// # Notes
/// - Tag paths are relative to `dir`, even during recursion.
/// - Hidden directories (starting with `.`) are skipped automatically.
/// - Case-insensitive folder matching is used for `ignored_folders`, but extension matching is case-sensitive.
///
/// # Example
/// ```rust
/// let dir = Path::new("src");
/// let exts = &["rs", "toml"];
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
                    match std::fs::read_to_string(&path) {
                        Ok(contents) => {
                            writeln!(output, "<{}>", rel_str)?;
                            writeln!(output, "{}", contents)?;
                            writeln!(output, "</{}>\n", rel_str)?;
                        }
                        Err(e) => {
                            eprintln!("⚠️ Skipping {:?}: {}", path, e);
                        }
                    }
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

/// Determines whether a file should be processed based on its extension.
///
/// # Purpose
/// Acts as a simple filter to determine whether a file is eligible for inclusion in output generation,
/// based solely on its extension. Used to avoid processing binary, unsupported, or irrelevant file types.
///
/// # Parameters
/// - `path`: A reference to the [`Path`] representing the file to evaluate.
/// - `valid_exts`: A slice of valid file extensions (`&[&str]`), such as `["rs", "json", "toml"]`.
///   Extensions **must not** include leading dots (e.g., `"rs"`, not `".rs"`).
///
/// # Returns
/// - `true` if the file has a valid extension (case-sensitive match).
/// - `false` if:
///   - The file has no extension,
///   - The extension cannot be interpreted as UTF-8,
///   - Or the extension is not in the list of `valid_exts`.
///
/// # Behavior
/// - Extracts the file extension using `Path::extension()`.
/// - Converts the extension to a string using `.to_str()`.
/// - Performs an exact string match against `valid_exts`.
///
/// # Panics
/// This function does **not** panic under normal conditions.
///
/// # Limitations
/// - Does not distinguish between binary/text content.
/// - Case-sensitive by design: `"RS"` is treated differently than `"rs"`.
/// - Does not inspect the actual content of the file or its MIME type.
///
/// # Use Cases
/// - Filtering which files should be read and wrapped in output.
/// - Used in both recursive and non-recursive modes for consistency.
///
/// # Example
/// ```rust
/// let path = Path::new("src/main.rs");
/// assert!(is_human_readable(path, &["rs", "txt"]));
///
/// let path = Path::new("README");
/// assert!(!is_human_readable(path, &["md"])); // has no extension
/// ```
///
/// # Future Enhancements
/// - Optionally support case-insensitive matching.
/// - Add content-based filtering (e.g., UTF-8 check or printable character ratio).
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}

/// Appends a block of custom user-provided text to the end of the output file,
/// under a `[Additional Commands]` header.
///
/// # Purpose
/// Allows users to supplement the main output (`tags_output.txt`) with additional
/// instructions, prompts, or commands that should be interpreted after the code sections.
///
/// # Parameters
/// - `file_path`: Path to the file to append to (typically `"tags_output.txt"`).
/// - `additional_commands`: Multiline string of user-defined commands or instructions to include.
///
/// # Behavior
/// - Opens the file in append mode, creating it if it does not exist.
/// - Writes a new section labeled `[Additional Commands]` followed by the content of `additional_commands`.
/// - Ensures newline padding before and after for readability and visual separation.
///
/// # Output Format
/// The appended section will resemble:
/// ```text
/// [Additional Commands]
/// user instructions...
/// ```
///
/// # Returns
/// - `Ok(())` if writing was successful.
/// - `Err(std::io::Error)` if the file could not be opened or written to.
///
/// # Panics
/// - This function does not panic. All I/O operations are fallible and propagated using `?`.
///
/// # Notes
/// - If `additional_commands` is empty, the section is still written unless filtered externally.
/// - Multiple calls will result in multiple `[Additional Commands]` sections unless deduplicated by the caller.
/// - No sanitization or validation is performed on the `additional_commands` text.
///
/// # Use Cases
/// - Appending detailed user instructions to aid post-processing of the file.
/// - Integrating command prompts or AI instructions for tools like ChatGPT.
///
/// # Example
/// ```rust
/// append_additional_commands("tags_output.txt", "TODO: Review all unwrap() usages.")?;
/// ```
///
/// # See Also
/// - [`write_folder_tags`]: Main output function that creates the initial `tags_output.txt`.
/// - [`copy_to_clipboard`]: Can be used after appending to share the result.
pub fn append_additional_commands(
    file_path: &str,
    additional_commands: &str,
) -> std::io::Result<()> {
    let mut file = File::options().create(true).append(true).open(file_path)?;

    writeln!(file, "\n[Additional Commands]")?;
    writeln!(file, "{}\n", additional_commands)?;

    Ok(())
}

/// Recursively traverses a directory and writes the contents of matching files to an output file,
/// wrapping each in XML-style tags based on its relative path.
///
/// # Purpose
/// Processes all files with specified extensions within a directory tree, skipping ignored or hidden folders,
/// and outputs their contents to a single file (`tags_output.txt`) with clear path-based XML tags.
///
/// # Parameters
/// - `root_dir`: The root directory of the traversal, used to compute relative paths for output tags.
/// - `dir`: The current directory being visited (initially the same as `root_dir`).
/// - `valid_exts`: List of file extensions (`&[&str]`) that are allowed (e.g., `["rs", "json"]`).
/// - `output`: A mutable reference to the output file (typically `tags_output.txt`).
/// - `ignored_folders`: A list of folder names (case-insensitive) to skip during traversal
///   (e.g., `[".git", "target"]`).
///
/// # Behavior
/// - Walks the directory tree rooted at `dir`, following folders recursively.
/// - Skips:
///   - Hidden directories (names starting with `.`).
///   - Directories matching any entry in `ignored_folders`, case-insensitively.
/// - For each file:
///   - If it matches a valid extension (`is_human_readable`), the file is opened and read as UTF-8.
///   - Its contents are written to `output`, surrounded by `<relative\path>` XML-style tags.
///
/// # Output Format
/// For each valid file:
/// ```xml
/// <subdir\file.rs>
/// // file contents...
/// </subdir\file.rs>
/// ```
///
/// # Returns
/// - `Ok(())` if all operations succeed.
/// - `Err(std::io::Error)` if a fatal error occurs (e.g., directory read or file open failure).
///
/// # Panics
/// - This function does not panic. All I/O errors are propagated or skipped with logging.
///
/// # Notes
/// - Invalid UTF-8 file paths or contents are skipped silently or with a warning to `stderr`.
/// - Tag paths always use Windows-style `\` separators for cross-platform consistency.
/// - Designed to integrate with `write_folder_tags`, not called directly by end users.
///
/// # Example
/// ```rust
/// let root = Path::new("src");
/// let mut output = File::create("tags_output.txt")?;
/// let ignored = vec!["target".to_string(), ".git".to_string()];
/// write_folder_tags_recursive(root, root, &["rs"], &mut output, &ignored)?;
/// ```
///
/// # See Also
/// - [`write_folder_tags`]: Top-level API that wraps this function and handles non-recursive behavior.
/// - [`is_human_readable`]: Checks extension validity before file content is read.
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
                    match std::fs::read_to_string(&path) {
                        Ok(contents) => {
                            writeln!(output, "<{}>", rel_str)?;
                            writeln!(output, "{}", contents)?;
                            writeln!(output, "</{}>\n", rel_str)?;
                        }
                        Err(e) => {
                            eprintln!("⚠️ Skipping {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
