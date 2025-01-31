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
use clipboard_win::{formats, Clipboard, Setter};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs::{read_to_string, OpenOptions};
use std::io;
use std::io::Write;
use std::path::PathBuf;

/// # Main Entry Point
///
/// The `main` function serves as the entry point for the `code-file-wrapper` program.
/// It is responsible for processing command-line arguments, invoking the appropriate
/// logic for file processing, and handling user interactions via the GUI if necessary.
///
/// ## Functionality
/// - Parses command-line arguments to check if a folder path is provided.
/// - If a path is provided, processes the files in the directory using `write_folder_tags`.
/// - If no path is provided, launches a GUI to allow directory and mode selection.
/// - Writes the tagged contents of files into `tags_output.txt`.
/// - Appends any additional user-provided commands to `tags_output.txt`.
/// - Optionally copies the contents of `tags_output.txt` to the clipboard.
///
/// ## Parameters
/// - This function does not take any parameters directly, but it uses command-line
///   arguments parsed through `CliArgs`.
///
/// ## Returns
/// - This function does not return a value. The process terminates with `std::process::exit(0)`
///   upon successful completion, or exits with an error code if an error occurs.
///
/// ## Error Handling
/// - If the provided folder path does not exist, an error message is displayed and the program exits.
/// - If `write_folder_tags` fails, an error is printed, and the program exits.
/// - If `append_additional_commands` fails, an error is printed, but execution continues.
/// - If clipboard copying fails, an error is printed, but execution continues.
///
/// ## Panics
/// - The function does not explicitly panic, but it will exit if critical errors occur, such as
///   failing to create or write to `tags_output.txt`.
///
/// ## Side Effects
/// - Creates (or overwrites) `tags_output.txt`.
/// - Writes the contents of selected files into `tags_output.txt`.
/// - Modifies the system clipboard if clipboard copying is enabled.
///
/// ## Usage
/// - Running the program with a folder path:
///   ```sh
///   code-file-wrapper /path/to/directory
///   ```
/// - Running the program without arguments (launches GUI):
///   ```sh
///   code-file-wrapper
///   ```
///
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

/// # Append Additional Commands to `tags_output.txt`
///
/// The `append_additional_commands` function appends user-specified text to the output file
/// (`tags_output.txt`). If no additional commands are provided, it still ensures that a section
/// header (`[Additional Commands]`) is included in the file.
///
/// ## Functionality
/// - Opens `tags_output.txt` in **append mode** (creating it if it does not exist).
/// - Writes a section header `[Additional Commands]`.
/// - Appends the user-provided additional commands, or a default message if none are provided.
///
/// ## Parameters
/// - `file_path: &str` → The path to the output file (`tags_output.txt`).
/// - `additional_commands: &str` → The user-provided text to append to the file.
///
/// ## Returns
/// - `std::io::Result<()>` → Returns `Ok(())` if the operation is successful, or an `Err` if
///   any file operation fails.
///
/// ## Error Handling
/// - If the file cannot be opened or created, an error is printed, and an `Err` is returned.
/// - If writing to the file fails, an error is printed, and an `Err` is returned.
///
/// ## Panics
/// - This function does not explicitly panic, but it will return an error if file operations fail.
///
/// ## Side Effects
/// - Modifies `tags_output.txt` by appending user-specified commands.
/// - Creates `tags_output.txt` if it does not exist.
///
/// ## Usage
/// ```rust
/// let file_path = "tags_output.txt";
/// let additional_commands = "TODO: Implement new parsing logic.";
/// if let Err(e) = append_additional_commands(file_path, additional_commands) {
///     eprintln!("Error writing additional commands: {}", e);
/// }
/// ```
///
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

/// Copies the contents of `tags_output.txt` to the system clipboard.
///
/// # Functionality
/// - Reads the contents of `tags_output.txt`.
/// - Copies the contents to the clipboard using the `clipboard-win` crate.
///
/// # Parameters
/// - `file_path: &str` → The path to the output file (`tags_output.txt`).
///
/// # Returns
/// - `std::io::Result<()>` → Returns `Ok(())` if the clipboard operation succeeds, or an `Err`
///   if reading the file or setting the clipboard fails.
///
/// # Error Handling
/// - If reading `tags_output.txt` fails, an error is returned.
/// - If setting the clipboard fails, an error is returned.
///
/// # Panics
/// - This function does not explicitly panic but will return an error if:
///   - The clipboard provider fails to initialize.
///   - The clipboard contents cannot be set.
///
/// # Side Effects
/// - Copies the contents of `tags_output.txt` to the system clipboard.
///
/// # Usage
/// ```rust
/// if let Err(e) = copy_to_clipboard("tags_output.txt") {
///     eprintln!("Error copying to clipboard: {}", e);
/// }
/// ```
fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    // Open clipboard and set contents
    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

/// Launches the mode selection GUI and returns the user's choices.
///
/// # Functionality
/// - Displays a graphical user interface (GUI) using `eframe` and `egui`.
/// - Allows the user to select a processing mode from a list of available options.
/// - Provides a checkbox to enable automatic clipboard saving.
/// - Includes a multi-line text input area for additional commands.
/// - Returns the selected mode, clipboard save preference, and additional commands input.
///
/// # Parameters
/// - `modes: Vec<&str>` → A vector of available processing mode names.
/// - `initial_pos: Option<(f32, f32)>` → An optional tuple specifying the initial window position.
///
/// # Returns
/// - `(Option<String>, bool, String)`
///   - `Option<String>` → The selected processing mode, or `None` if the user did not select one.
///   - `bool` → `true` if the user enabled clipboard saving, `false` otherwise.
///   - `String` → The user-provided additional commands.
///
/// # Error Handling
/// - This function does not return errors directly. If the GUI fails to launch, the program will continue execution.
///
/// # Panics
/// - This function does not explicitly panic but will exit if the `eframe::run_native` function fails.
///
/// # Side Effects
/// - Opens a GUI window for user interaction.
/// - The selected processing mode and user input are stored in mutable references.
///
/// # Usage
/// ```rust
/// let modes = vec!["Rust", "JSON", "XML"];
/// let cursor_position = Some((100.0, 200.0));
/// let (selected_mode, enable_clipboard, additional_commands) = mode_selection_gui(modes, cursor_position);
///
/// if let Some(mode) = selected_mode {
///     println!("User selected mode: {}", mode);
/// }
/// if enable_clipboard {
///     println!("Clipboard saving enabled.");
/// }
/// println!("Additional Commands: {}", additional_commands);
/// ```
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

/// Retrieves the current mouse cursor position as a tuple `(x, y)`.
///
/// # Functionality
/// - Uses the Windows API to obtain the current cursor position on the screen.
/// - Returns the position as floating-point coordinates.
///
/// # Returns
/// - `Option<(f32, f32)>`
///   - `Some((x, y))` → If the cursor position is successfully retrieved.
///   - `None` → If the cursor position cannot be obtained.
///
/// # Error Handling
/// - This function does not return errors but will return `None` if the Windows API call fails.
///
/// # Panics
/// - This function does not explicitly panic but relies on unsafe Rust code for the Windows API call.
///
/// # Side Effects
/// - Calls the Windows API function `GetCursorPos`, which may fail in restricted environments.
///
/// # Platform-Specific Behavior
/// - This function is only implemented for Windows. It will not compile on non-Windows platforms without modification.
///
/// # Usage
/// ```rust
/// if let Some((x, y)) = get_cursor_position() {
///     println!("Cursor position: ({}, {})", x, y);
/// } else {
///     println!("Failed to retrieve cursor position.");
/// }
/// ```
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
