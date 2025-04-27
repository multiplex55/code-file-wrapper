//! # Utilities Module
//!
//! This module provides helper functions for clipboard operations
//! and retrieving the cursor position.

use clipboard_win::{formats, Clipboard, Setter};
use std::fs::read_to_string;
use std::io;

/// Copies the contents of a specified file into the system clipboard as Unicode text.
///
/// # Parameters
/// - `file_path`: A string slice (`&str`) representing the path to the file whose contents should be copied.
///
/// # Returns
/// - `Ok(())`: If the file is successfully read and its contents are set into the clipboard.
/// - `Err(std::io::Error)`: If any I/O or clipboard operation fails.
///
/// # Behavior
/// - Reads the entire contents of the specified file into a `String`.
/// - Attempts to open the system clipboard with up to 10 retries (to handle contention with other applications).
/// - Writes the file's contents into the clipboard in Unicode (UTF-16) format.
/// - If any step fails (file read, clipboard open, or clipboard write), returns an error.
///
/// # Error Handling
/// - If reading the file fails (e.g., file not found or permission denied), returns an `Err`.
/// - If the clipboard cannot be accessed after multiple attempts, returns an `Err`.
/// - If writing Unicode content to the clipboard fails, returns an `Err`.
/// - Errors are wrapped as `std::io::Error` for consistent I/O-like error handling.
///
/// # Panics
/// - This function does not explicitly panic under normal conditions.
/// - Panics could only occur if the underlying clipboard library (`clipboard-win`) encounters an unrecoverable internal error, which is highly unlikely.
///
/// # Example
/// ```rust
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     copy_to_clipboard("tags_output.txt")?;
///     println!("File contents successfully copied to clipboard.");
///     Ok(())
/// }
/// ```
///
/// # Platform-Specific Behavior
/// - **Supported**: Only on Windows, using the `clipboard-win` crate.
/// - **Unsupported**: On Linux, macOS, or other platforms; would require alternative clipboard implementations.
///
/// # Notes
/// - Clipboard writes overwrite any existing clipboard contents.
/// - If the file is empty, the clipboard will be cleared to an empty string.
/// - Retrying clipboard access helps avoid common issues where another application temporarily locks the clipboard.
/// - Unicode encoding ensures compatibility with international characters (e.g., Chinese, Japanese, accented characters).
pub fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

/// Retrieves the current mouse cursor position on the screen as an `(x, y)` tuple.
///
/// # Returns
/// - `Some((x, y))`: If the cursor position is successfully retrieved.
///   - `x`: Horizontal screen coordinate (in pixels).
///   - `y`: Vertical screen coordinate (in pixels).
/// - `None`: If the underlying Windows API call fails.
///
/// # Behavior
/// - Calls the Windows API function `GetCursorPos` to fetch the global screen coordinates of the cursor.
/// - Converts the raw integer coordinates (`i32`) to `f32` for compatibility with GUI frameworks and scaling.
/// - Returns the coordinates wrapped in `Some` if successful; otherwise, returns `None`.
///
/// # Error Handling
/// - If the call to `GetCursorPos` fails (e.g., due to permission issues, rare on normal desktops), returns `None`.
/// - No panics or crashes occur on failure; the function degrades gracefully.
///
/// # Panics
/// - This function does not panic under normal circumstances.
/// - Only a catastrophic failure within the `windows` crate (Win32 bindings) could cause an indirect panic.
///
/// # Example
/// ```rust
/// fn main() {
///     match get_cursor_position() {
///         Some((x, y)) => println!("Cursor is at: ({}, {})", x, y),
///         None => eprintln!("Failed to retrieve cursor position."),
///     }
/// }
/// ```
///
/// # Platform-Specific Behavior
/// - **Supported**: Only on Windows, using `windows` crate's bindings to `user32.dll`.
/// - **Unsupported**: On non-Windows platforms (Linux, macOS); alternative implementations would be needed for cross-platform support.
///
/// # Notes
/// - The returned coordinates are relative to the full screen (not relative to any window).
/// - Values are in physical screen pixels; additional scaling (for DPI awareness) may be necessary for certain GUI contexts.
/// - The function assumes that calling `GetCursorPos` is cheap enough to use per-frame in GUI applications.
pub fn get_cursor_position() -> Option<(f32, f32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut point).is_ok() {
            return Some((point.x as f32, point.y as f32));
        }
    }
    None
}
