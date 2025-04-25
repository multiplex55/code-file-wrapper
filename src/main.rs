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
use std::process::Command;

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

    let (selected_dir, selected_mode, enable_clipboard_copy, additional_commands, preset_texts) =
        mode_selection_gui(mode_list, cursor_position);

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

    if let Err(e) = write_folder_tags(&dir, valid_exts) {
        eprintln!("❌ ERROR: Could not write folder tags: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = append_additional_commands("tags_output.txt", &additional_commands) {
        eprintln!("❌ ERROR: Could not append additional commands: {}", e);
    }

    for preset_text in preset_texts {
        if let Err(e) = append_additional_commands("tags_output.txt", &preset_text) {
            eprintln!("❌ ERROR: Could not append preset text: {}", e);
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

/// - `Option<PathBuf>`: The directory selected by the user.
/// - `Option<String>`: The file type mode selected by the user.
/// - `bool`: A boolean indicating whether clipboard copy is enabled.
/// - `String`: Additional commands entered by the user.
/// - `Vec<String>`: A vector of preset command texts selected by the user.
///
/// # Behavior
/// - The GUI provides file type selection buttons dynamically sorted alphabetically.
/// - The user can select a directory and file type for processing.
/// - Additional commands and preset commands can be added to be included in the output file.
/// - The GUI will be positioned at the cursor location if available, otherwise, a default position `(100.0, 100.0)` is used.
///
/// # Panics
/// - This function does not explicitly panic but will terminate the application if the GUI encounters an unrecoverable error.
///
/// # Example Usage
/// ```rust
/// let modes = vec!["AHK", "Rust", "JSON"];
/// let cursor_position = get_cursor_position();
/// let (dir, mode, clipboard, commands, presets) = mode_selection_gui(modes, cursor_position);
/// ```
///
/// # Notes
/// - The GUI is implemented using `egui` and launched via `eframe::run_native`.
/// - The function blocks execution until the user closes the GUI.
/// - Selected options are returned for further processing in `main.rs`.
fn mode_selection_gui(
    modes: Vec<&str>,
    initial_pos: Option<(f32, f32)>,
) -> (Option<PathBuf>, Option<String>, bool, String, Vec<String>) {
    let mut selected_mode: Option<String> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();
    let mut selected_dir: Option<PathBuf> = None;
    let mut preset_texts = Vec::new();

    // Retrieve cursor position if available
    let (x, y) = initial_pos.unwrap_or((100.0, 100.0)); // Default if position is unavailable

    let app = ModeSelector::new(
        modes,
        &mut selected_mode,
        &mut enable_clipboard_copy,
        &mut additional_commands,
        &mut selected_dir,
        &mut preset_texts,
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
    )
}
