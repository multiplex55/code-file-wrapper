//! # Main Entry Point
//!
//! This is the main executable for the `code-file-wrapper` program.
//! It handles command-line argument parsing, GUI interaction, and file operations.

mod file_ops;
mod gui;
mod presets;
mod utils;

use crate::file_ops::{append_additional_commands, write_folder_tags};
use crate::gui::ModeSelector;
use crate::utils::{copy_to_clipboard, get_cursor_position};

use eframe::egui;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::collections::HashMap;
use std::path::PathBuf;

/// The main entry point of the `code-file-wrapper` program.
///
/// # Overview
/// This function initializes the program, retrieves user selections via the GUI,
/// processes selected files based on their extensions, and writes the formatted output
/// to `tags_output.txt`. Additionally, it can copy the output to the clipboard if
/// requested by the user.
///
/// # Functionality
/// - Retrieves the cursor position to open the GUI at the cursor's location.
/// - Displays a GUI allowing the user to select:
///   - A directory containing the files.
///   - A mode corresponding to file extensions.
///   - Whether to copy the output to the clipboard.
///   - Additional commands to append to the output.
/// - Reads and filters files from the selected directory based on the selected mode.
/// - Writes the tagged file contents to `tags_output.txt`.
/// - Appends additional commands and preset texts if selected.
/// - Optionally copies the output to the clipboard.
///
/// # Panics
/// - This function will terminate the process (`std::process::exit(1)`) if:
///   - No directory is selected.
///   - The selected path is not a valid directory.
///   - Writing to `tags_output.txt` fails.
///
/// # Errors
/// - If appending additional commands or copying to the clipboard fails, an error message is printed to `stderr`,
///   but the program will continue execution.
///
/// # Side Effects
/// - Creates `tags_output.txt` in the current working directory.
/// - Opens a graphical user interface for mode selection.
/// - Can modify the system clipboard contents.
fn main() {
    let modes: HashMap<&str, Vec<&str>> = HashMap::from([
        ("AHK", vec!["ahk"]),
        ("Rust", vec!["rs"]),
        ("JSON", vec!["json"]),
        ("XML", vec!["xml"]),
        ("C/CPP", vec!["c", "cpp", "h"]),
        ("lua", vec!["lua"]),
    ]);

    let cursor_position = get_cursor_position();

    let mut mode_list: Vec<&str> = modes.keys().cloned().collect();
    mode_list.sort_unstable(); // Sort the modes alphabetically

    let (
        selected_dir,
        selected_mode,
        enable_clipboard_copy,
        additional_commands,
        preset_texts,
        enable_recursive_search,
        ignored_folders,
    ) = mode_selection_gui(mode_list, cursor_position);

    let Some(dir) = selected_dir else {
        eprintln!("⚠️ No directory selected. Exiting.");
        std::process::exit(0);
    };

    println!("📂 User selected directory: {:?}", dir);

    if !dir.is_dir() {
        eprintln!("❌ ERROR: Selected path is not a directory.");
        std::process::exit(1);
    }

    let Some(valid_exts) = selected_mode.and_then(|mode| modes.get(mode.as_str())) else {
        eprintln!("⚠️ No mode selected. Exiting.");
        std::process::exit(0);
    };

    let ignored_folders: Vec<String> = ignored_folders
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if let Err(e) = write_folder_tags(&dir, valid_exts, enable_recursive_search, &ignored_folders) {
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

/// Launches a graphical user interface (GUI) that allows the user to select a directory, file type mode,
/// and various additional options, returning the selections upon closing.
///
/// # Parameters
/// - `modes`: A `Vec<&str>` listing available file type modes that the user can choose from (e.g., "Rust", "JSON").
/// - `initial_pos`: An `Option<(f32, f32)>` specifying the initial screen coordinates (x, y) for the GUI window.
///   - If `None`, the window will default to a position at (100.0, 100.0).
///
/// # Returns
/// A tuple containing:
/// - `Option<PathBuf>`: The path to the selected directory, if one was chosen.
/// - `Option<String>`: The selected file type mode, if one was chosen.
/// - `bool`: Whether the user enabled automatic clipboard copying.
/// - `String`: Additional commands entered by the user (may be empty).
/// - `Vec<String>`: A vector of preset texts selected by the user (may be empty).
/// - `bool`: Whether recursive directory search was enabled.
/// - `String`: Multiline string listing folder names to ignore (one per line, case insensitive).
///
/// # Behavior
/// - Instantiates a new `ModeSelector` struct, passing mutable references to shared state variables.
/// - Configures `eframe::NativeOptions` for the GUI window:
///   - Sets inner size constraints.
///   - Applies minimum allowed window dimensions.
///   - Positions the window based on `initial_pos`, if provided.
/// - Launches the GUI via `eframe::run_native`.
/// - Execution is blocked until the GUI window is closed.
/// - Upon closure, the function returns the final state of all selection values.
///
/// # Error Handling
/// - If `eframe::run_native` fails internally, the error is ignored (`let _ = ...`).
///   - This is safe because the project expects the GUI to run in a normal user environment.
/// - No runtime validation is done inside this function; validation is handled inside the `ModeSelector`'s `update` method.
///
/// # Panics
/// - This function does not explicitly panic.
/// - Panics may occur only if `eframe` itself encounters a fatal error, which is rare under normal conditions.
///
/// # Example
/// ```rust
/// let modes = vec!["Rust", "JSON", "AHK"];
/// let cursor_pos = get_cursor_position();
///
/// let (dir, mode, clipboard, additional_commands, presets, recursive, ignored_folders) =
///     mode_selection_gui(modes, cursor_pos);
/// ```
///
/// # Notes
/// - This function separates GUI concerns cleanly from the main program logic.
/// - If no directory or mode is selected (due to GUI cancellation), returned values will reflect that via `None`.
/// - The GUI fields are reset to empty/defaults before each new GUI launch.
/// - This function should generally only be called once at program startup.
fn mode_selection_gui(
    modes: Vec<&str>,
    initial_pos: Option<(f32, f32)>,
) -> (
    Option<PathBuf>,
    Option<String>,
    bool,
    String,
    Vec<String>,
    bool,
    String,
) {
    let mut selected_mode: Option<String> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();
    let mut selected_dir: Option<PathBuf> = None;
    let mut preset_texts = Vec::new();
    let mut enable_recursive_search = false;
    let mut ignored_folders = String::new();

    // Retrieve cursor position if available
    let (x, y) = initial_pos.unwrap_or((100.0, 100.0)); // Default if position is unavailable

    let app = ModeSelector::new(
        modes,
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
        selected_dir,
        selected_mode,
        enable_clipboard_copy,
        additional_commands,
        preset_texts,
        enable_recursive_search,
        ignored_folders,
    )
}
