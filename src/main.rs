//! # Main Entry Point
//!
//! Coordinates program execution, launching the GUI and processing user-selected files.
//!
//! # Responsibilities
//! - Starts the GUI and gathers user input.
//! - Validates directory and mode selections.
//! - Initiates file reading, filtering, and output generation.
//! - Optionally copies the final result to the clipboard or opens it in Notepad.
//!
//! # Key Functions
//! - [`main`]: The primary entry point. Launches the app, orchestrates all major steps.
//! - [`mode_selection_gui`]: Wrapper that runs the GUI and returns user selections.
//!
//! # Dependencies
//! - Relies on `file_ops` for tag generation.
//! - Relies on `gui` for user input.
//! - Relies on `utils` for clipboard and cursor behavior.
//! - Relies on `presets` for saved preset data.
//!
//! # Output
//! - Generates or updates `tags_output.txt`.

#![cfg_attr(windows, windows_subsystem = "windows")]
mod file_ops;
mod filetypes;
mod gui;
mod presets;
mod utils;

use crate::file_ops::{append_additional_commands, write_folder_tags};
use crate::filetypes::{get_filetypes, FileTypeGroup};
use crate::gui::ModeSelector;
use crate::utils::{copy_to_clipboard, get_cursor_position};

use eframe::egui;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::path::PathBuf;

/// The main entry point of the application.
///
/// # Purpose
/// Coordinates the entire program flow, from GUI input to file processing and clipboard interaction.
///
/// # Parameters
/// None.
///
/// # Behavior
/// - Loads initial file type groups from disk (or uses defaults).
/// - Displays the GUI to collect user input (directory, file type, recursion, etc.).
/// - Validates the user input and exits with a warning if invalid.
/// - Calls [`write_folder_tags`] to process matching files in the selected directory.
/// - Appends any user-supplied commands and preset commands to the output file.
/// - Copies the file to the clipboard if requested, or optionally opens it in Notepad.
///
/// # Panics
/// - Does not explicitly panic, but will `exit(1)` if:
///   - The selected path is not a directory.
///   - `tags_output.txt` could not be written.
///
/// # Errors
/// - Errors are logged to `stderr`, including:
///   - Invalid directory selection.
///   - File write failures.
///   - Clipboard failures or inability to launch Notepad.
///
/// # Side Effects
/// - Overwrites or creates `tags_output.txt` in the working directory.
/// - Optionally modifies the system clipboard.
/// - Optionally spawns Notepad (`notepad.exe`) to view the output.
///
/// # Notes
/// - Runs only on Windows (clipboard and dialog support).
/// - Relies on GUI state to be collected before file operations.
/// - Uses structured JSON files for storing user presets and file type modes.
/// - Always exits cleanly via `std::process::exit(0)` or `exit(1)`.
///
/// # Example
/// ```rust
/// fn main() {
///     // Triggers GUI, processes files, writes output, etc.
/// }
/// ```
///
/// # Related
/// - [`mode_selection_gui`] – Launches the GUI and gathers input.
/// - [`write_folder_tags`] – Handles directory traversal and XML tag output.
/// - [`append_additional_commands`] – Adds user/preset commands to the final output.
/// - [`copy_to_clipboard`] – Sends the output to the Windows clipboard.
fn main() {
    let initial_file_type_groups = get_filetypes();
    let cursor_position = get_cursor_position();

    let (
        file_type_groups,
        selected_dir,
        selected_type_index,
        enable_clipboard_copy,
        additional_commands,
        preset_texts,
        enable_recursive_search,
        ignored_folders,
    ) = mode_selection_gui(initial_file_type_groups.clone(), cursor_position);

    let Some(dir) = selected_dir else {
        eprintln!("⚠️ No directory selected. Exiting.");
        std::process::exit(0);
    };

    println!("📂 User selected directory: {:?}", dir);

    if !dir.is_dir() {
        eprintln!("❌ ERROR: Selected path is not a directory.");
        std::process::exit(1);
    }

    let Some(group) = selected_type_index.and_then(|i| file_type_groups.get(i)) else {
        eprintln!("⚠️ No file type group selected. Exiting.");
        std::process::exit(0);
    };

    let valid_exts: Vec<&str> = group.extensions.iter().map(String::as_str).collect();

    let ignored_folders: Vec<String> = ignored_folders
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if let Err(e) = write_folder_tags(&dir, &valid_exts, enable_recursive_search, &ignored_folders)
    {
        eprintln!("❌ ERROR: Could not write folder tags: {}", e);
        std::process::exit(1);
    }

    let mut combined_additional = String::new();

    for preset_text in &preset_texts {
        combined_additional.push('\n');
        combined_additional.push_str(preset_text.trim());
        combined_additional.push('\n');
    }

    if !additional_commands.trim().is_empty() {
        combined_additional.push('\n');
        combined_additional.push_str(additional_commands.trim());
        combined_additional.push('\n');
    }

    if !combined_additional.trim().is_empty() {
        if let Err(e) = append_additional_commands("tags_output.txt", &combined_additional) {
            eprintln!(
                "❌ ERROR: Could not append combined additional commands: {}",
                e
            );
        }
    }

    if enable_clipboard_copy {
        if let Err(e) = copy_to_clipboard("tags_output.txt") {
            eprintln!("❌ ERROR: Could not copy to clipboard: {}", e);
        }
    } else {
        let result = MessageDialog::new()
            .set_title("Open Output File?")
            .set_description("Would you like to open the generated tags_output.txt file?")
            .set_buttons(MessageButtons::YesNo)
            .set_level(MessageLevel::Info)
            .show();

        if result == MessageDialogResult::Yes {
            if let Err(e) = std::process::Command::new("notepad")
                .arg("tags_output.txt")
                .spawn()
            {
                eprintln!("❌ ERROR: Failed to open tags_output.txt: {}", e);
            }
        }
    }

    std::process::exit(0);
}

/// Launches the graphical user interface and returns user selections for file processing.
///
/// # Purpose
/// Gathers user configuration via an interactive GUI, including:
/// - Target directory
/// - File type group (by extension)
/// - Whether to copy results to the clipboard
/// - Additional instructional or preset commands
/// - Recursive folder scanning options and ignored folders
///
/// # Parameters
/// - `file_type_groups`: A list of [`FileTypeGroup`] values used to populate the file type dropdown.
/// - `initial_pos`: Optional screen coordinates `(x, y)` to position the GUI window near the cursor.
///
/// # Returns
/// A tuple containing:
/// - `Vec<FileTypeGroup>`: Possibly updated list of file type groups (if the user modified them).
/// - `Option<PathBuf>`: The selected directory path.
/// - `Option<usize>`: Index of the selected file type group, or `None` if not selected.
/// - `bool`: Whether to copy output to the clipboard after generation.
/// - `String`: Custom text commands to be appended to the output file.
/// - `Vec<String>`: Collected preset command texts selected by the user.
/// - `bool`: Whether recursive directory search is enabled.
/// - `String`: Newline-separated list of folder names to ignore (e.g., `"target\n.git"`).
///
/// # Behavior
/// - Spawns an `eframe` GUI using [`ModeSelector`], blocking until user presses OK or closes the window.
/// - Captures all user interaction and returns configuration as pure values.
/// - Defaults the GUI position to `(100.0, 100.0)` if no cursor position is provided.
///
/// # Panics
/// - This function does not panic.
/// - If `eframe::run_native()` fails, no error is raised — a default empty configuration is returned.
///
/// # Side Effects
/// - Opens a GUI window.
/// - Writes no files, only collects data.
///
/// # Notes
/// - GUI layout and logic are fully encapsulated in `ModeSelector::update`.
/// - If the user closes the window without making selections, the returned directory and mode are `None`.
///
/// # Example
/// ```rust
/// let groups = get_filetypes();
/// let cursor = get_cursor_position();
/// let (groups, dir, mode, clipboard, commands, presets, recursive, ignored) =
///     mode_selection_gui(groups, cursor);
/// ```
///
/// # Related
/// - [`ModeSelector`] – Core GUI logic and layout.
/// - [`get_cursor_position`] – Used to determine where to place the GUI window.
fn mode_selection_gui(
    file_type_groups: Vec<FileTypeGroup>,
    initial_pos: Option<(f32, f32)>,
) -> (
    Vec<FileTypeGroup>,
    Option<PathBuf>, // selected_dir
    Option<usize>,   // selected file type group index
    bool,            // clipboard
    String,          // additional commands
    Vec<String>,     // preset texts
    bool,            // recursive
    String,          // ignored folders
) {
    let mut local_file_type_groups = file_type_groups;
    let mut selected_mode: Option<usize> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();
    let mut selected_dir: Option<PathBuf> = None;
    let mut preset_texts = Vec::new();
    let mut enable_recursive_search = false;
    let mut ignored_folders = String::new();

    // Retrieve cursor position if available
    let (x, y) = initial_pos.unwrap_or((100.0, 100.0)); // Default if position is unavailable

    let app = ModeSelector::new(
        &mut local_file_type_groups,
        &mut selected_mode,
        &mut enable_clipboard_copy,
        &mut additional_commands,
        &mut selected_dir,
        &mut preset_texts,
        &mut enable_recursive_search,
        &mut ignored_folders,
        false,
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0])
            .with_min_inner_size([500.0, 350.0])
            .with_position([x, y]), // Set GUI position to cursor location
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Select Processing Mode",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    );

    (
        local_file_type_groups,
        selected_dir,
        selected_mode,
        enable_clipboard_copy,
        additional_commands,
        preset_texts,
        enable_recursive_search,
        ignored_folders,
    )
}
