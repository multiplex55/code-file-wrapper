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
use std::collections::HashMap;
use std::path::PathBuf;

/// Main function that orchestrates the program execution.
fn main() {
    let modes: HashMap<&str, Vec<&str>> = HashMap::from([
        ("AHK", vec!["ahk"]),
        ("Rust", vec!["rs"]),
        ("JSON", vec!["json"]),
        ("XML", vec!["xml"]),
        ("C/CPP", vec!["c", "cpp", "h"]),
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
    }

    std::process::exit(0);
}

/// Launches the mode selection GUI at the cursor position.
///
/// # Returns
/// - `selected_dir`: The directory chosen by the user.
/// - `selected_mode`: The selected file type.
/// - `enable_clipboard_copy`: Whether to copy output to the clipboard.
/// - `additional_commands`: Any additional commands entered by the user.
/// - `preset_texts`: The texts associated with selected preset buttons.
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
