//! # File Operations Module
//!
//! This module manages reading file contents, filtering based on extensions,
//! and writing them into `tags_output.txt` in an XML-like format.

use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::Path;

/// Writes the contents of all valid files in a specified directory to a `tags_output.txt` file,
/// wrapping each file's contents inside XML-like tags based on the file path.
///
/// # Parameters
/// - `dir`: A reference to the root `Path` to scan for files.
/// - `valid_exts`: A slice of string slices (`&[&str]`) that lists allowed file extensions.
/// - `recursive`: A boolean flag indicating whether to search directories recursively.
/// - `ignored_folders`: A slice of `String` values specifying folder names to ignore (case-insensitive).
///
/// # Returns
/// - `Ok(())`: If all file processing and writing operations complete successfully.
/// - `Err(std::io::Error)`: If any filesystem or I/O operations fail during the process.
///
/// # Behavior
/// - Creates or overwrites the file `tags_output.txt` in the current working directory.
/// - If `recursive` is true, traverses the directory tree recursively, skipping ignored folders.
/// - Otherwise, processes only the top-level directory.
/// - For each valid file:
///   - Reads its contents (assumes UTF-8 encoding).
///   - Writes the contents between XML-style tags named after the relative file path.
/// - Appends a set of instructional comments at the end of the file.
///
/// # Error Handling
/// - If `tags_output.txt` cannot be created or written to, the function returns an error.
/// - If any individual file cannot be read, a warning is printed to `stderr`, but processing continues.
/// - Fatal errors (e.g., inability to read the directory) immediately propagate as `Err`.
///
/// # Panics
/// - This function does not explicitly panic.
/// - Relies on the `?` operator for error propagation; any panics would stem from library bugs or OS failures.
///
/// # Example
/// ```rust
/// use std::path::Path;
///
/// fn main() -> std::io::Result<()> {
///     let dir = Path::new("src");
///     let valid_exts = &["rs", "txt"];
///     let recursive = true;
///     let ignored_folders = vec!["target".to_string(), ".git".to_string()];
///
///     write_folder_tags(dir, valid_exts, recursive, &ignored_folders)?;
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - Non-UTF-8 files will cause a warning and will not be included.
/// - Folder ignoring is case-insensitive.
/// - Hidden folders (starting with ".") are automatically ignored during recursion.
/// - The output file is always named `tags_output.txt`, and is overwritten at each invocation.
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
    writeln!(output, "* Provide context above and below code changes.")?;
    writeln!(
        output,
        "* Be extremely explicit with where to make changes."
    )?;

    Ok(())
}

/// Checks whether a given file has a valid human-readable extension as defined by a provided list.
///
/// # Parameters
/// - `path`: A reference to a `Path` representing the file to check.
/// - `valid_exts`: A slice of string slices (`&[&str]`) containing the allowed file extensions (without leading dots).
///
/// # Returns
/// - `true` if the file has an extension matching one of the entries in `valid_exts`.
/// - `false` if the file has no extension, or the extension is not recognized as valid.
///
/// # Behavior
/// - Extracts the extension from the file's path.
/// - Converts the extension into a UTF-8 string slice.
/// - Compares it against the provided list of valid extensions.
/// - Returns a boolean indicating whether the file should be considered "readable."
///
/// # Error Handling
/// - This function does not return any `Result`; it is infallible.
/// - If the file has no extension or if the extension is invalid Unicode, the function simply returns `false`.
///
/// # Panics
/// - This function does not panic under normal operating conditions.
///
/// # Example
/// ```rust
/// use std::path::Path;
///
/// let file = Path::new("example.rs");
/// let valid_extensions = &["rs", "txt", "json"];
///
/// assert_eq!(is_human_readable(file, valid_extensions), true);
/// ```
///
/// # Notes
/// - Only the file extension is checked; no attempt is made to inspect the file's contents.
/// - File extensions are compared exactly as provided, so extension case sensitivity must be considered if needed.
/// - This function treats a lack of extension (e.g., "README") as non-readable (`false`).
/// - Binary files may still have allowed extensions and would incorrectly be considered "human-readable" by this function.
///
/// # Future Improvements
/// - To better detect true human-readability, content-based heuristics (like trying to parse text) could be added.
///   However, such improvements would come at a performance cost and are outside the scope of this simple check.
fn is_human_readable(path: &Path, valid_exts: &[&str]) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return valid_exts.contains(&ext);
    }
    false
}

/// Appends a block of additional text to an existing file, creating the file if it does not exist.
///
/// # Parameters
/// - `file_path`: A string slice (`&str`) representing the path to the file where the text will be appended.
/// - `additional_commands`: A string slice (`&str`) containing the text content to append after the header.
///
/// # Returns
/// - `Ok(())`: If the file is successfully opened (or created) and the content is appended without errors.
/// - `Err(std::io::Error)`: If the file could not be opened, created, or written to.
///
/// # Behavior
/// - Opens the target file in append mode (`OpenOptions::append(true)`).
/// - If the file does not exist, it will be created.
/// - Appends the header line `[Additional Commands]` followed by the user-provided `additional_commands` text.
/// - Ensures a newline is added after the appended text for proper formatting.
///
/// # Error Handling
/// - Propagates any I/O errors encountered during file operations via the `?` operator.
/// - If writing fails (e.g., due to permission issues), returns an error immediately.
///
/// # Panics
/// - This function does not explicitly panic under normal conditions.
/// - Only severe OS or file system-level failures could trigger a panic indirectly (e.g., allocation failure).
///
/// # Example
/// ```rust
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     append_additional_commands("tags_output.txt", "TODO: Refactor error handling.")?;
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - Repeated calls to this function will result in multiple `[Additional Commands]` headers being appended.
/// - If `additional_commands` is empty, only the header will be appended.
/// - No deduplication or merging logic is applied; entries are written exactly as provided.
/// - Writing operations are atomic at the OS level per line, but concurrent writes from multiple threads or processes are not synchronized.
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
/// - `root_dir`: A reference to the root directory `Path` against which relative file paths are computed.
/// - `dir`: A reference to the current directory `Path` to process (initially same as `root_dir`).
/// - `valid_exts`: A slice of string slices (`&[&str]`) listing valid file extensions to include.
/// - `output`: A mutable reference to an already opened `File` where output will be written.
/// - `ignored_folders`: A slice of `String` values listing folder names to skip (case-insensitive).
///
/// # Returns
/// - `Ok(())`: If traversal and writing complete without any fatal errors.
/// - `Err(std::io::Error)`: If a filesystem or I/O error occurs that prevents further processing.
///
/// # Behavior
/// - Recursively walks the directory tree rooted at `dir`.
/// - Skips any subdirectories matching names in `ignored_folders` (case-insensitive) or names starting with a `.` (hidden folders).
/// - For each file that has a valid extension:
///   - Reads the file contents (assuming UTF-8 encoding).
///   - Writes the contents into the `output` file, wrapped in XML-like tags based on the relative path from `root_dir`.
/// - Continues processing even if individual files fail to read; only traversal-level errors abort the function.
///
/// # Error Handling
/// - Traversal errors like an unreadable directory immediately return an `Err`.
/// - Individual file read failures are ignored silently — only successfully readable files are processed.
/// - Uses the `?` operator extensively for propagation of fatal I/O errors.
///
/// # Panics
/// - No explicit panics; standard filesystem operations are used safely with error propagation.
///
/// # Example
/// ```rust
/// use std::path::Path;
/// use std::fs::File;
///
/// fn main() -> std::io::Result<()> {
///     let root_dir = Path::new("src");
///     let valid_exts = &["rs", "txt"];
///     let ignored = vec!["target".to_string(), ".git".to_string()];
///     let mut output = File::create("tags_output.txt")?;
///
///     write_folder_tags_recursive(root_dir, root_dir, valid_exts, &mut output, &ignored)?;
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - Folder names are lowercased before comparison to handle case-insensitivity.
/// - Hidden directories (those beginning with `.`) are automatically excluded without needing to list them in `ignored_folders`.
/// - The function relies on the caller to ensure that `output` is opened in the desired mode (e.g., truncating or appending).
/// - Large directory trees may result in significant memory and I/O usage.
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
