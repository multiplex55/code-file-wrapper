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
//! - Relies on `generation` for tag generation orchestration.
//! - Relies on `gui` for user input.
//! - Relies on `utils` for clipboard and cursor behavior.
//! - Relies on `presets` for saved preset data.
//!
//! # Output
//! - Generates or updates the selected output file.

mod cli;
mod file_ops;
mod filetypes;
mod generation;
mod gui;
mod presets;
mod profiles;
mod utils;

use crate::cli::{build_run_request, Cli, Command};
use crate::filetypes::{get_filetypes, FileTypeGroup};
use crate::generation::{generate_tag_output, GenerationSummary, TagGenerationRequest};
use crate::gui::ModeSelector;
use crate::presets::get_presets;
use crate::profiles::{
    delete_profile, find_profile, load_profiles, profile_from_run_args, profile_to_run_request,
    save_profile,
};
use crate::utils::get_cursor_position;

use clap::Parser;
use eframe::egui;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::path::PathBuf;

#[cfg(windows)]
const OPEN_COMMAND: &str = "notepad";
#[cfg(not(windows))]
const OPEN_COMMAND: &str = "xdg-open";

/// The main entry point of the application.
///
/// Parses command-line arguments first. With no subcommand, or with `gui`, it
/// launches the existing graphical flow. Other subcommands run non-interactive
/// CLI tasks and exit with an appropriate status code.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Command::Gui) => run_gui_flow(),
        Some(Command::ListFileTypes) => list_file_types(),
        Some(Command::ListProfiles) => list_profiles(),
        Some(Command::DeleteProfile { name }) => {
            if let Err(error) = delete_profile(&name) {
                eprintln!("❌ ERROR: {error}");
                std::process::exit(1);
            }
            println!("✅ Deleted profile '{name}'.");
            std::process::exit(0);
        }
        Some(Command::SaveProfile(args)) => {
            let file_type_groups = get_filetypes();
            let presets = get_presets();
            let profile = match profile_from_run_args(args.name, args.run) {
                Ok(profile) => profile,
                Err(error) => {
                    eprintln!("❌ ERROR: {error}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = profile_to_run_request(&profile, &file_type_groups, &presets) {
                eprintln!("❌ ERROR: {error}");
                std::process::exit(1);
            }

            let profile_name = profile.name.clone();
            if let Err(error) = save_profile(profile, args.force) {
                eprintln!("❌ ERROR: {error}");
                std::process::exit(1);
            }
            println!("✅ Saved profile '{profile_name}'.");
            std::process::exit(0);
        }
        Some(Command::RunProfile { name }) => run_profile_command(&name),
        Some(Command::Run(args)) => {
            let file_type_groups = get_filetypes();
            let presets = get_presets();
            let built = match build_run_request(args, &file_type_groups, &presets) {
                Ok(built) => built,
                Err(error) => {
                    eprintln!("❌ ERROR: {error}");
                    std::process::exit(1);
                }
            };

            run_built_request(built);
        }
    }
}

fn run_profile_command(name: &str) -> ! {
    let profiles = load_profiles();
    let Some(profile) = find_profile(&profiles, name) else {
        eprintln!("❌ ERROR: Profile '{name}' does not exist.");
        std::process::exit(1);
    };

    let file_type_groups = get_filetypes();
    let presets = get_presets();
    let built = match profile_to_run_request(profile, &file_type_groups, &presets) {
        Ok(built) => built,
        Err(error) => {
            eprintln!("❌ ERROR: {error}");
            std::process::exit(1);
        }
    };

    run_built_request(built);
}

fn run_built_request(built: crate::cli::BuiltRunRequest) -> ! {
    let open_after = built.request.open_after;
    let extensions_used = built.extensions_used.clone();
    let summary = match generate_tag_output(built.request) {
        Ok(summary) => summary,
        Err(e) => {
            eprintln!("❌ ERROR: Could not generate tag output: {}", e);
            std::process::exit(1);
        }
    };

    if open_after {
        open_output_file(&summary);
    }

    print_cli_summary(&summary, &extensions_used);
    std::process::exit(0);
}

fn run_gui_flow() {
    let initial_file_type_groups = get_filetypes();
    let cursor_position = get_cursor_position();

    let (
        file_type_groups,
        selected_dir,
        selected_type_index,
        enable_clipboard_copy,
        output_path,
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

    let Some(group) = selected_type_index.and_then(|i| file_type_groups.get(i)) else {
        eprintln!("⚠️ No file type group selected. Exiting.");
        std::process::exit(0);
    };

    let ignored_folders: Vec<String> = ignored_folders
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let open_after = !enable_clipboard_copy;
    let request = TagGenerationRequest {
        root_dir: dir,
        extensions: group.extensions.clone(),
        recursive: enable_recursive_search,
        ignored_folders,
        output_path: PathBuf::from(output_path),
        additional_commands,
        preset_texts,
        copy_to_clipboard: enable_clipboard_copy,
        open_after,
    };

    let summary = match generate_tag_output(request) {
        Ok(summary) => summary,
        Err(e) => {
            eprintln!("❌ ERROR: Could not generate tag output: {}", e);
            std::process::exit(1);
        }
    };

    if open_after {
        let result = MessageDialog::new()
            .set_title("Open Output File?")
            .set_description(format!(
                "Would you like to open the generated output file?\n\n{}",
                summary.output_path.display()
            ))
            .set_buttons(MessageButtons::YesNo)
            .set_level(MessageLevel::Info)
            .show();

        if result == MessageDialogResult::Yes {
            open_output_file(&summary);
        }
    }

    std::process::exit(0);
}

fn list_profiles() {
    let profiles = load_profiles();
    if profiles.is_empty() {
        println!("No profiles saved.");
        return;
    }

    for profile in profiles {
        let file_type = profile.file_type.as_deref().unwrap_or("custom extensions");
        println!(
            "{}: dir={} file-type={} output={}",
            profile.name,
            profile.dir.display(),
            file_type,
            profile.output.display()
        );
    }
}

fn list_file_types() {
    for group in get_filetypes() {
        println!("{}: {}", group.name, group.extensions.join(", "));
    }
}

fn print_cli_summary(summary: &GenerationSummary, extensions_used: &[String]) {
    println!("✅ Generation complete.");
    println!("Output path: {}", summary.output_path.display());
    println!("Files included: {}", summary.files_written);
    println!("Files skipped: {}", summary.files_skipped);
    println!("Non-UTF8 files skipped: {}", summary.skipped_non_utf8_files);
    println!("Recursive: {}", summary.recursive);
    println!("Extensions used: {}", extensions_used.join(", "));
}

fn open_output_file(summary: &GenerationSummary) {
    if let Err(e) = std::process::Command::new(OPEN_COMMAND)
        .arg(&summary.output_path)
        .spawn()
    {
        eprintln!("❌ ERROR: Failed to open output file: {}", e);
    }
}

/// Launches the graphical user interface and returns user selections for file processing.
///
/// # Purpose
/// Gathers user configuration via an interactive GUI, including:
/// - Target directory
/// - File type group (by extension)
/// - Whether to copy results to the clipboard
/// - Output file path
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
/// - `String`: Output file path.
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
/// let (groups, dir, mode, clipboard, output, commands, presets, recursive, ignored) =
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
    String,          // output path
    String,          // additional commands
    Vec<String>,     // preset texts
    bool,            // recursive
    String,          // ignored folders
) {
    let mut local_file_type_groups = file_type_groups;
    let mut selected_mode: Option<usize> = None;
    let mut enable_clipboard_copy = false;
    let mut output_path = "tags_output.txt".to_string();
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
        &mut output_path,
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
        output_path,
        additional_commands,
        preset_texts,
        enable_recursive_search,
        ignored_folders,
    )
}
