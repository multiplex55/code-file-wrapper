//! # Utilities Module
//!
//! This module provides supporting functionality used throughout the application,
//! primarily for system-level tasks such as clipboard interaction and cursor positioning.
//!
//! # Contents
//! - [`copy_to_clipboard`]: Copies the contents of a file to the Windows clipboard as Unicode text.
//! - [`get_cursor_position`]: Retrieves the current global mouse cursor position (screen coordinates).
//!
//! # Platform Compatibility
//! - This module is **Windows-only** due to its use of the Win32 API (`clipboard-win` and `windows` crates).
//!
//! # Use Cases
//! - GUI positioning based on current cursor location.
//! - Copying output (`tags_output.txt`) to the clipboard for easy pasting into external tools (e.g., chatbots, editors).
//!
//! # Notes
//! - Functions in this module are designed to fail gracefully and never panic.
//! - They are safe to call from both GUI and CLI contexts.

use clipboard_win::{formats, Clipboard, Setter};
use std::fs::read_to_string;
use std::io;

/// Copies the contents of a file into the Windows system clipboard as Unicode text.
///
/// # Parameters
/// - `file_path`: The path to the file whose contents should be copied to the clipboard.
///
/// # Returns
/// - `Ok(())` if the file was successfully read and its contents placed into the clipboard.
/// - `Err(std::io::Error)` if:
///   - The file could not be read,
///   - The clipboard could not be opened,
///   - Or writing to the clipboard failed.
///
/// # Behavior
/// - Reads the entire file as a UTF-8 string.
/// - Opens the clipboard using `clipboard-win`, retrying up to 10 times (to account for potential contention).
/// - Writes the contents to the clipboard in Unicode (UTF-16) format.
///
/// # Panics
/// - This function does not panic under normal conditions.
/// - Internal panics may only occur if the `clipboard-win` crate encounters a critical system error (extremely rare).
///
/// # Notes
/// - This function is **Windows-only**. It uses the `clipboard-win` crate, which wraps native Win32 clipboard APIs.
/// - If the file is empty, the clipboard will be set to an empty string.
/// - Any previous clipboard contents will be overwritten.
/// - Retrying clipboard access helps avoid issues where another app (like a browser or editor) temporarily locks it.
///
/// # Limitations
/// - No support for non-text formats (e.g., images or rich text).
/// - No concurrent access protection — do not call this from multiple threads simultaneously.
///
/// # Example
/// ```rust
/// copy_to_clipboard("tags_output.txt")?;
/// println!("Copied tags_output.txt to clipboard.");
/// ```
pub fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

/// Retrieves the current position of the mouse cursor on the screen.
///
/// # Returns
/// - `Some((x, y))`: A tuple of screen coordinates (in pixels) if successful:
///   - `x`: Horizontal screen position.
///   - `y`: Vertical screen position.
/// - `None`: If the call to the Windows API fails.
///
/// # Behavior
/// - Uses the Win32 API function `GetCursorPos` via the `windows` crate to get the global screen coordinates.
/// - Converts the result from `i32` to `f32` for compatibility with UI libraries like `egui`.
/// - The coordinates are absolute (relative to the screen), not relative to any window or control.
///
/// # Platform Support
/// - **Supported**: Windows only.
/// - **Unsupported**: Will not compile on non-Windows systems unless conditional compilation is added.
///
/// # Panics
/// - This function does **not** panic.
/// - A failure to retrieve the cursor position results in `None`.
///
/// # Use Cases
/// - Used to position GUI windows (e.g., opening a UI near the cursor).
/// - Useful for tooltip systems, context menus, or floating windows.
///
/// # Example
/// ```rust
/// if let Some((x, y)) = get_cursor_position() {
///     println!("Cursor is at: ({x}, {y})");
/// } else {
///     eprintln!("⚠️ Could not retrieve cursor position.");
/// }
/// ```
///
/// # Notes
/// - Returns coordinates in physical screen pixels — no DPI scaling is applied.
/// - In multi-monitor setups, coordinates reflect the full desktop space and may be negative if the primary screen is not at (0,0).
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
