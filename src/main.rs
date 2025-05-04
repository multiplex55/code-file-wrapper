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

#![windows_subsystem = "windows"]
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

/// The main entry point of the `code-file-wrapper` application.
///
/// # Overview
/// This function initializes and orchestrates the core application logic,
/// including GUI interaction, directory traversal, file filtering, output generation,
/// and optional clipboard copying or user prompt to open the result.
///
/// # Behavior
/// - Launches a GUI to gather user input for:
///   - Target directory
///   - Desired file type/mode
///   - Optional recursive search toggle
///   - Preset and custom command texts
///   - Clipboard copy option
/// - Based on the selected file mode, filters for specific file extensions.
/// - Traverses the selected directory, optionally recursively.
/// - Writes file contents to a `tags_output.txt` file wrapped in XML-style tags.
/// - Appends preset commands and user-provided instructions if any.
/// - Copies the output to clipboard or optionally opens it in Notepad.
///
/// # Panics
/// - If the selected directory is missing or invalid, the program will terminate via `std::process::exit(1)`.
/// - Similarly, failure to write to `tags_output.txt` also causes an immediate process exit.
///
/// # Errors
/// - Warnings and recoverable errors (e.g., failure to copy to clipboard or open Notepad)
///   are printed to `stderr`, but do not crash the application.
///
/// # Side Effects
/// - Creates or overwrites `tags_output.txt` in the working directory.
/// - May update the clipboard (on Windows).
/// - Opens a GUI using the `eframe` crate and optionally spawns `notepad.exe`.
///
/// # Notes
/// - The user is required to make both a directory and file mode selection before proceeding.
/// - Folder ignore logic is handled case-insensitively.
/// - All user-facing dialogs are handled through the `rfd` crate.
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

/// Launches a graphical user interface that allows the user to select a directory, file type mode,
/// and various configuration options for processing files.
///
/// # Parameters
/// - `modes`: A `Vec<&str>` containing the list of selectable file type modes (e.g., `"Rust"`, `"JSON"`).
/// - `initial_pos`: An `Option<(f32, f32)>` specifying the screen coordinates to position the GUI window near the cursor.
///   - If `None`, defaults to `(100.0, 100.0)` on screen.
///
/// # Returns
/// A tuple containing:
/// - `Option<PathBuf>`: The selected directory (or `None` if canceled).
/// - `Option<String>`: The selected file type mode (or `None` if canceled).
/// - `bool`: Whether to automatically copy the output to the clipboard.
/// - `String`: Additional command text entered by the user.
/// - `Vec<String>`: List of preset texts selected or composed during the session.
/// - `bool`: Whether recursive directory scanning is enabled.
/// - `String`: Multiline text representing folders to ignore (one per line).
///
/// # Behavior
/// - Constructs a `ModeSelector` instance, passing mutable references to shared state variables.
/// - Configures GUI options using `eframe::NativeOptions`, including size, minimum bounds, and screen position.
/// - Launches a blocking GUI session with `eframe::run_native`, which continues until the user presses the `OK` button or closes the window.
/// - Once the GUI closes, the final state is returned to the caller.
///
/// # Panics
/// - This function does not panic under expected conditions.
/// - If `eframe::run_native` fails internally (e.g., failed window creation), the error is ignored and defaults are returned.
///
/// # Side Effects
/// - Opens a native GUI window using `egui`.
/// - Collects user input and stores it in memory (shared mutable fields passed into `ModeSelector`).
///
/// # Notes
/// - No runtime validation is performed inside this function; it is handled in `ModeSelector::update()`.
/// - If the user closes the GUI without making a selection, `None` values will be returned for mode and directory.
/// - This function should be called early in `main()` before any file processing occurs.
///
/// # Example
/// ```rust
/// let modes = vec!["Rust", "JSON", "AHK"];
/// let cursor_pos = get_cursor_position();
///
/// let (dir, mode, clipboard, additional, presets, recursive, ignored) =
///     mode_selection_gui(modes, cursor_pos);
///
/// if let Some(path) = dir {
///     println!("Selected directory: {:?}", path);
/// }
/// ```
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
