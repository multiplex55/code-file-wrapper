//! # GUI Module
//!
//! This module manages the graphical user interface (GUI) using `egui`.
//! It allows users to select a directory, file type, and preset commands.

use crate::presets::{get_presets, PresetCommand};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashSet;
use std::path::PathBuf;

/// GUI State Management
pub struct ModeSelector<'a> {
    modes: Vec<String>,
    selected_mode: &'a mut Option<String>,
    enable_clipboard_copy: &'a mut bool,
    additional_commands: &'a mut String,
    selected_dir: &'a mut Option<PathBuf>,
    preset_texts: &'a mut Vec<String>,
    selected_presets: HashSet<String>,
    warning_message: String,
    presets: Vec<PresetCommand>,
    enable_recursive_search: &'a mut bool,
    ignored_folders: &'a mut String,
}

impl<'a> ModeSelector<'a> {
    /// Creates a new GUI instance.
    pub fn new(
        modes: Vec<&str>,
        selected_mode: &'a mut Option<String>,
        enable_clipboard_copy: &'a mut bool,
        additional_commands: &'a mut String,
        selected_dir: &'a mut Option<PathBuf>,
        preset_texts: &'a mut Vec<String>,
        enable_recursive_search: &'a mut bool,
        ignored_folders: &'a mut String,
    ) -> Self {
        Self {
            modes: modes.into_iter().map(String::from).collect(),
            selected_mode,
            enable_clipboard_copy,
            additional_commands,
            selected_dir,
            preset_texts,
            selected_presets: HashSet::new(),
            warning_message: String::new(),
            presets: get_presets(),
            enable_recursive_search,
            ignored_folders,
        }
    }
}

impl eframe::App for ModeSelector<'_> {
    /// Updates the graphical user interface (GUI) each frame, allowing users to interact with directory,
    /// mode, and preset selections.
    ///
    /// # Parameters
    /// - `ctx`: A reference to the `egui::Context`, representing the current GUI state and context.
    /// - `_frame`: A mutable reference to the `eframe::Frame`, representing the window/frame state (unused).
    ///
    /// # Behavior
    /// - Displays interactive widgets for:
    ///   - Selecting a directory.
    ///   - Selecting a file type (mode).
    ///   - Enabling or disabling automatic clipboard copy.
    ///   - Enabling or disabling recursive search.
    ///   - Specifying folders to ignore during recursion.
    ///   - Entering additional command text.
    ///   - Selecting preset commands to be appended to output.
    /// - Provides an `OK` button that validates selections and closes the GUI if inputs are valid.
    /// - Displays a warning message in red text if required selections are missing.
    /// - Updates internal shared mutable state (`self.selected_mode`, `self.selected_dir`, etc.) as the user interacts.
    ///
    /// # Layout Structure
    /// 1. **Directory Selection Row**:
    ///    - "Select Directory" button opens a folder picker dialog.
    ///    - Displays selected directory path in a disabled text field.
    ///
    /// 2. **File Type (Mode) Selection**:
    ///    - Presents a button for each available file type mode.
    ///    - Highlights the selected mode with a green background.
    ///
    /// 3. **Clipboard Copy and Recursive Search Options**:
    ///    - Checkboxes to enable clipboard copying and recursive folder search.
    ///
    /// 4. **Ignored Folders Entry** *(visible only if recursive search is enabled)*:
    ///    - Multi-line text field allowing entry of folders to ignore (one per line).
    ///
    /// 5. **Additional Commands Input**:
    ///    - Scrollable, resizable text box for arbitrary user commands.
    ///
    /// 6. **Preset Commands Section**:
    ///    - Button for each preset command with a tooltip preview.
    ///    - Selected presets are highlighted.
    ///
    /// 7. **Warning Messages**:
    ///    - Displays a red warning if validation fails (e.g., missing directory or mode).
    ///
    /// 8. **OK Button**:
    ///    - When clicked:
    ///      - Validates that a directory and a mode have been selected.
    ///      - If validation passes, selected preset texts are collected and appended.
    ///      - Closes the GUI window via `ctx.send_viewport_cmd(egui::ViewportCommand::Close)`.
    ///
    /// # Error Handling
    /// - No explicit error propagation; errors are indicated to the user via in-GUI warning messages.
    /// - Folder selection failure simply leaves the selection unchanged (no crash).
    ///
    /// # Panics
    /// - This function does not explicitly panic under normal circumstances.
    /// - Underlying failures in GUI framework (eframe/egui) could cause unexpected panics, but are very unlikely.
    ///
    /// # Example
    /// ```rust
    /// fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ///     egui::CentralPanel::default().show(ctx, |ui| {
    ///         // Directory selection button and preview
    ///         if ui.button("Select Directory").clicked() {
    ///             // ...
    ///         }
    ///         // Additional UI elements...
    ///     });
    /// }
    /// ```
    ///
    /// # Notes
    /// - Folder names for ignore are treated as case-insensitive later in the processing stage.
    /// - File mode selection tooltips show which extensions are handled by each mode.
    /// - Visual styling uses explicit background color changes for selected items to aid usability.
    /// - GUI position defaults near the cursor but is handled externally (`mode_selection_gui`).
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Directory Selection
            ui.horizontal(|ui| {
                if ui.button("Select Directory").clicked() {
                    if let Some(dir) = FileDialog::new().set_directory(".").pick_folder() {
                        *self.selected_dir = Some(dir);
                        self.warning_message.clear();
                    }
                }
                if let Some(dir) = &self.selected_dir {
                    ui.add(
                        egui::TextEdit::singleline(&mut dir.display().to_string())
                            .desired_width(300.0)
                            .interactive(false),
                    );
                }
            });

            ui.label("File Type Selection:");
            ui.horizontal_wrapped(|ui| {
                for mode in &self.modes {
                    let is_selected = self.selected_mode.as_deref() == Some(mode);
                    let default_bg = ui.visuals().widgets.inactive.bg_fill;
                    let default_text = ui.visuals().widgets.inactive.fg_stroke.color;

                    let (bg_color, text_color) = if is_selected {
                        (egui::Color32::from_rgb(0, 120, 40), egui::Color32::WHITE)
                    } else {
                        (default_bg, default_text)
                    };

                    let button = ui.add(
                        egui::Button::new(egui::RichText::new(mode).color(text_color))
                            .fill(bg_color),
                    );

                    if button.clicked() {
                        *self.selected_mode = Some(mode.clone());
                        self.warning_message.clear();
                    }

                    button.on_hover_text(format!("Includes files with extensions: {:?}", mode));
                }
            });

            // Clipboard Save Option
            ui.checkbox(
                self.enable_clipboard_copy,
                "Enable save to clipboard automatically",
            );

            // Recursive Search Option
            ui.checkbox(
                self.enable_recursive_search,
                "Enable recursive directory search",
            );

            // Ignored Folders TextArea

            if *self.enable_recursive_search {
                ui.group(|ui| {
                    ui.label("Ignore Folders (one per line, case insensitive):");
                    egui::ScrollArea::vertical()
                        .id_salt("ignore_folders_scrollarea")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(self.ignored_folders)
                                    .desired_width(ui.available_width())
                                    .desired_rows(4)
                                    .clip_text(false),
                            );
                        });
                });
            }

            // Additional Commands Box
            ui.group(|ui| {
                ui.label("Additional Commands:");
                egui::ScrollArea::both()
                    .id_salt("additional_commands_scrollarea")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(self.additional_commands)
                                .desired_width(ui.available_width())
                                .desired_rows(5)
                                .clip_text(false),
                        );
                    });
            });

            // Preset Command Selection with Tooltips
            ui.label("Preset Commands:");
            ui.horizontal_wrapped(|ui| {
                for preset in &self.presets {
                    let is_selected = self.selected_presets.contains(preset.name);
                    let default_bg = ui.visuals().widgets.inactive.bg_fill;
                    let default_text = ui.visuals().widgets.inactive.fg_stroke.color;

                    let (bg_color, text_color) = if is_selected {
                        (egui::Color32::from_rgb(0, 120, 40), egui::Color32::WHITE)
                    } else {
                        (default_bg, default_text)
                    };

                    let button = ui.add(
                        egui::Button::new(egui::RichText::new(preset.name).color(text_color))
                            .fill(bg_color),
                    );

                    if button.clicked() {
                        if is_selected {
                            self.selected_presets.remove(preset.name);
                        } else {
                            self.selected_presets.insert(preset.name.to_string());
                        }
                    }

                    button.on_hover_text(format!("Will add: \n\n\"{}\"", preset.text));
                }
            });

            ui.separator();

            // Display Warning Message (if any)
            if !self.warning_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.warning_message);
            }

            // OK Button with Validation
            if ui.button("OK").clicked() {
                if self.selected_dir.is_none() {
                    self.warning_message =
                        "⚠️ Please select a directory before proceeding!".to_string();
                } else if self.selected_mode.is_none() {
                    self.warning_message =
                        "⚠️ Please select a file type before proceeding!".to_string();
                } else {
                    // Append selected presets when GUI closes
                    for preset in &self.presets {
                        if self.selected_presets.contains(preset.name) {
                            self.preset_texts.push(preset.text.to_string());
                        }
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
    }
}
