//! # Utilities Module
//!
//! This module provides helper functions for clipboard operations
//! and retrieving the cursor position.

use clipboard_win::{formats, Clipboard, Setter};
use std::fs::read_to_string;
use std::io;

/// Copies the contents of `tags_output.txt` to the system clipboard.
///
/// # Parameters
/// - `file_path`: Path to the output file.
///
/// # Returns
/// - `Ok(())` if successful, otherwise an `Err`.
pub fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

/// Retrieves the current mouse cursor position as a `(x, y)` tuple.
///
/// # Returns
/// - `Some((x, y))` if successful.
/// - `None` if the cursor position cannot be obtained.
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
