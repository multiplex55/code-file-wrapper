//! # Main Entry Point
//!
//! This file is the main executable for the `code-file-wrapper` program. It
//! reads command line arguments, optionally opens a folder picker dialog, and
//! then generates an output file containing tag-wrapped file contents.

mod cli;
mod file_ops;

use crate::cli::CliArgs;
use crate::file_ops::write_folder_tags;
use clap::Parser;
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::path::PathBuf;

/// Main entry point for `code-file-wrapper`.
///
/// If a valid path is provided, that folder is immediately processed.
/// Otherwise, a folder picker dialog is opened to allow folder selection,
/// and a mode selection GUI is presented.
fn main() {
    // Define available modes and their corresponding file extensions
    let modes: HashMap<&str, Vec<&str>> = HashMap::from([
        ("AHK", vec!["ahk"]),
        ("Rust", vec!["rs"]),
        ("JSON", vec!["json"]),
        ("XML", vec!["xml"]),
        ("C/CPP", vec!["c", "cpp", "h"]),
    ]);

    // Parse command-line arguments
    let args = CliArgs::parse();

    // Determine folder path
    let folder = if let Some(ref path) = args.path {
        let p = PathBuf::from(path);
        if !p.exists() {
            eprintln!("Provided path does not exist. Exiting.");
            std::process::exit(1);
        }
        p
    } else {
        // Open a folder picker dialog if no path is provided
        let selected_dir = FileDialog::new().set_directory(".").pick_folder();

        let Some(dir) = selected_dir else {
            eprintln!("No directory selected. Exiting.");
            std::process::exit(0);
        };

        if !dir.is_dir() {
            eprintln!("Selected path is not a directory. Exiting.");
            std::process::exit(1);
        }

        // Get mouse cursor position for initial window placement
        let cursor_position = get_cursor_position();

        // Launch the mode selection GUI
        let selected_mode = mode_selection_gui(modes.keys().cloned().collect(), cursor_position);
        let Some(valid_exts) = selected_mode.and_then(|mode| modes.get(mode.as_str())) else {
            eprintln!("No mode selected. Exiting.");
            std::process::exit(0);
        };

        if let Err(e) = write_folder_tags(&dir, valid_exts) {
            eprintln!("Error creating tags: {e}");
            std::process::exit(1);
        }

        // Exit since work is done here
        std::process::exit(0);
    };

    // Default behavior if path is provided
    if let Err(e) = write_folder_tags(&folder, &["ini", "txt", "rs", "cs", "json", "xml", "ahk"]) {
        eprintln!("Error creating tags: {e}");
        std::process::exit(1);
    }
    std::process::exit(0);
}

/// Launches the mode selection GUI and returns the selected mode.
fn mode_selection_gui(modes: Vec<&str>, initial_pos: Option<(f32, f32)>) -> Option<String> {
    let mut selected_mode: Option<String> = None;

    let app = ModeSelector::new(modes, &mut selected_mode);

    // Set the initial position of the GUI window
    let mut native_options = eframe::NativeOptions::default();

    if let Some((x, y)) = initial_pos {
        native_options.viewport = native_options
            .viewport
            .with_position(egui::pos2(x, y))
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 200.0]);
    }

    let _ = eframe::run_native(
        "Select Processing Mode",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    );

    selected_mode
}

/// State and behavior for the mode selection GUI.
struct ModeSelector<'a> {
    modes: Vec<String>,
    selected_mode: &'a mut Option<String>,
}

impl<'a> ModeSelector<'a> {
    /// Creates a new ModeSelector with the given modes.
    pub fn new(modes: Vec<&str>, selected_mode: &'a mut Option<String>) -> Self {
        Self {
            modes: modes.into_iter().map(String::from).collect(),
            selected_mode,
        }
    }
}

impl eframe::App for ModeSelector<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Select a Processing Mode");

            // Create a grid of buttons for each mode
            egui::Grid::new("mode_selection_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    for mode in &self.modes {
                        if ui.button(mode).clicked() {
                            *self.selected_mode = Some(mode.clone());
                            // frame.close(); // Close the GUI
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

/// Retrieves the current mouse cursor position as `(x, y)` coordinates.
fn get_cursor_position() -> Option<(f32, f32)> {
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
