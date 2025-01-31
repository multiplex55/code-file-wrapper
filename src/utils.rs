//! # Utilities Module
//!
//! This module provides helper functions for clipboard operations
//! and retrieving the cursor position.

use clipboard_win::{formats, Clipboard, Setter};
use std::fs::read_to_string;
use std::io;

/// Copies the contents of a specified file to the system clipboard.
///
/// # Parameters
/// - `file_path`: A string slice (`&str`) representing the path to the file whose contents should be copied.
///
/// # Returns
/// - `Ok(())` if the file contents are successfully copied to the clipboard.
/// - `Err(std::io::Error)` if an error occurs during file reading or clipboard operations.
///
/// # Behavior
/// - Reads the entire contents of the file specified by `file_path`.
/// - Opens the system clipboard and writes the file contents as Unicode text.
/// - If the clipboard is unavailable, an error is returned.
///
/// # Error Handling
/// - If the file cannot be opened or read, an `Err(std::io::Error)` is returned.
/// - If the clipboard cannot be accessed, an `Err(std::io::Error)` is returned.
/// - If writing to the clipboard fails, an `Err(std::io::Error)` is returned.
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
///
///     match copy_to_clipboard(file_path) {
///         Ok(_) => println!("File contents copied to clipboard."),
///         Err(e) => eprintln!("Failed to copy to clipboard: {}", e),
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - The clipboard is attempted to be accessed up to 10 times before failing.
/// - The function writes Unicode text, ensuring compatibility with different languages and character sets.
/// - If the file is empty, an empty string will be copied to the clipboard.
/// - Some clipboard managers may clear the clipboard contents after a short duration.
///
/// # Platform-Specific Considerations
/// - This function is implemented using the `clipboard-win` crate, which is specific to Windows.
/// - On other operating systems, an alternative clipboard management library is required.
pub fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

/// Retrieves the current mouse cursor position as a tuple of `(x, y)` coordinates.
///
/// # Returns
/// - `Some((x, y))` if the cursor position is successfully retrieved, where:
///   - `x`: The horizontal coordinate of the cursor.
///   - `y`: The vertical coordinate of the cursor.
/// - `None` if the cursor position could not be obtained.
///
/// # Behavior
/// - Uses the Windows API function `GetCursorPos` to fetch the current cursor position.
/// - Converts the raw coordinate values into `f32` for better compatibility with graphical interfaces.
///
/// # Error Handling
/// - If `GetCursorPos` fails, the function returns `None`, indicating that the cursor position could not be retrieved.
///
/// # Panics
/// - This function does not panic under normal conditions.
///
/// # Example Usage
/// ```rust
/// if let Some((x, y)) = get_cursor_position() {
///     println!("Cursor is at: ({}, {})", x, y);
/// } else {
///     eprintln!("Failed to retrieve cursor position.");
/// }
/// ```
///
/// # Platform-Specific Considerations
/// - This function relies on the `windows` crate and the Windows API.
/// - It is only supported on Windows; alternative implementations are required for other operating systems.
///
/// # Notes
/// - The coordinates are relative to the screen, not to any specific window.
/// - The returned values may need further transformation when used in GUI applications with scaling or multiple monitors.
/// - This function does not provide additional information such as which window the cursor is over.
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
