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
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Main entry point for `code-file-wrapper`.
///
/// If a valid path is provided, that folder is immediately processed.
/// Otherwise, a folder picker dialog is opened to allow folder selection,
/// and a mode selection GUI is presented.
fn main() {
    let modes: HashMap<&str, Vec<&str>> = HashMap::from([
        ("AHK", vec!["ahk"]),
        ("Rust", vec!["rs"]),
        ("JSON", vec!["json"]),
        ("XML", vec!["xml"]),
        ("C/CPP", vec!["c", "cpp", "h"]),
    ]);

    let args = CliArgs::parse();
    let folder: PathBuf;

    if let Some(ref path) = args.path {
        folder = PathBuf::from(path);
        if !folder.exists() {
            eprintln!("❌ ERROR: Provided path does not exist.");
            std::process::exit(1);
        }

        println!("📂 Processing folder: {:?}", folder);

        if let Err(e) = write_folder_tags(
            &folder,
            &[
                "ini", "txt", "rs", "cs", "json", "xml", "ahk", "c", "cpp", "h",
            ],
        ) {
            eprintln!("❌ ERROR: Could not process files: {}", e);
            std::process::exit(1);
        }

        if let Err(e) = append_additional_commands("tags_output.txt", "") {
            eprintln!("❌ ERROR: Could not append additional commands: {}", e);
        }

        std::process::exit(0);
    } else {
        let selected_dir = FileDialog::new().set_directory(".").pick_folder();
        let Some(dir) = selected_dir else {
            eprintln!("⚠️ No directory selected. Exiting.");
            std::process::exit(0);
        };

        println!("📂 User selected directory: {:?}", dir);

        if !dir.is_dir() {
            eprintln!("❌ ERROR: Selected path is not a directory.");
            std::process::exit(1);
        }

        let cursor_position = get_cursor_position();
        let (selected_mode, enable_clipboard_copy, additional_commands) =
            mode_selection_gui(modes.keys().cloned().collect(), cursor_position);

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

        if enable_clipboard_copy {
            if let Err(e) = copy_to_clipboard("tags_output.txt") {
                eprintln!("❌ ERROR: Could not copy to clipboard: {}", e);
            }
        }

        std::process::exit(0);
    }
}

/// Appends additional commands to `tags_output.txt`. If empty, it still writes the section header.
fn append_additional_commands(file_path: &str, additional_commands: &str) -> std::io::Result<()> {
    // Debug print statement to verify function execution
    println!("Appending additional commands to {}", file_path);

    // Open file in append mode, create if it does not exist
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // Ensure the section header is always written
    writeln!(file, "\n[Additional Commands]")?;

    // Append the additional commands if available
    if !additional_commands.trim().is_empty() {
        writeln!(file, "{}\n", additional_commands)?;
    } else {
        writeln!(file, "No additional commands provided.\n")?;
    }

    println!("Successfully appended additional commands.");
    Ok(())
}

/// Copies the contents of `tags_output.txt` to the clipboard.
fn copy_to_clipboard(file_path: &str) -> std::io::Result<()> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Clipboard access failed"))?;

    let file_contents = read_to_string(file_path)?;
    ctx.set_contents(file_contents).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to set clipboard contents",
        )
    })
}

/// Launches the mode selection GUI and returns the selected mode.
fn mode_selection_gui(
    modes: Vec<&str>,
    initial_pos: Option<(f32, f32)>,
) -> (Option<String>, bool, String) {
    let mut selected_mode: Option<String> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();

    let app = ModeSelector::new(
        modes,
        &mut selected_mode,
        &mut enable_clipboard_copy,
        &mut additional_commands,
    );

    // Set initial position of GUI window
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

    (selected_mode, enable_clipboard_copy, additional_commands)
}

/// State and behavior for the mode selection GUI.
struct ModeSelector<'a> {
    modes: Vec<String>,
    selected_mode: &'a mut Option<String>,
    enable_clipboard_copy: &'a mut bool,
    additional_commands: &'a mut String,
}

impl<'a> ModeSelector<'a> {
    /// Creates a new ModeSelector with the given modes.
    pub fn new(
        modes: Vec<&str>,
        selected_mode: &'a mut Option<String>,
        enable_clipboard_copy: &'a mut bool,
        additional_commands: &'a mut String,
    ) -> Self {
        Self {
            modes: modes.into_iter().map(String::from).collect(),
            selected_mode,
            enable_clipboard_copy,
            additional_commands,
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
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.end_row();
                    }
                });

            // Checkbox for enabling clipboard copy
            ui.checkbox(
                self.enable_clipboard_copy,
                "Enable save to clipboard automatically",
            );

            // Large, scrollable text input area for additional commands
            ui.label("Additional Commands:");
            ui.add(
                egui::TextEdit::multiline(self.additional_commands)
                    .desired_width(380.0) // Make it wider
                    .desired_rows(10) // Display more lines before scrolling
                    .clip_text(false), // Ensure scrolling instead of clipping
            );
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
