//! # GUI Module
//!
//! Provides the graphical interface for selecting directories, file types, and preset commands.
//!
//! # Purpose
//! - Allows users to configure how the application processes files.
//! - Enables selection of presets, additional instructions, and clipboard behavior.
//! - Offers management tools to create, edit, and delete command presets.
//!
//! # Key Components
//! - [`ModeSelector`]: The main GUI application state, managing all interactive elements.
//! - `eframe::egui`: Used to build and render the interface.
//! - `rfd`: Used for native folder selection dialogs.
//!
//! # Features
//! - Directory and file mode selection.
//! - Recursive search toggle with folder ignore input.
//! - Additional commands input (multiline).
//! - Preset command dropdown with editing capabilities.
//! - Warning and success messages inline in the UI.
//!
//! # Behavior
//! - Uses `eframe::run_native` to block execution until the user completes the selection.
//! - Updates shared mutable state passed by reference from `main.rs`.

use crate::presets::save_presets;
use crate::presets::{get_presets, PresetCommand};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashSet;
use std::path::PathBuf;

/// Holds the interactive state and logic for the main GUI window.
///
/// # Purpose
/// - Centralizes all state used during the GUI session.
/// - Tracks user selections like mode, directory, clipboard settings, ignored folders, and presets.
/// - Manages the lifecycle and interactions of the preset manager popup.
///
/// # Fields
/// - `modes`: All available file type modes (e.g., Rust, JSON).
/// - `selected_mode`: Currently selected file type mode (shared mutable).
/// - `enable_clipboard_copy`: Whether the clipboard should be updated after output is generated.
/// - `additional_commands`: Multiline string entered by the user to append to the output.
/// - `selected_dir`: The selected folder path for file processing.
/// - `preset_texts`: Output accumulator for selected presets (shared mutable).
/// - `selected_presets`: Set of selected preset names (limited to 1 in the current UI).
/// - `warning_message`: Message shown in red if validation fails (e.g., no directory selected).
/// - `presets`: Full list of loaded/editable `PresetCommand` objects.
/// - `enable_recursive_search`: Whether to search directories recursively (shared mutable).
/// - `ignored_folders`: Textbox input for folder names to skip (shared mutable).
/// - `open_manage_presets`: Whether the preset manager window is currently open.
/// - `open_preset_index`: Index of the currently expanded preset panel (if any).
/// - `success_message`: Temporary success toast used when saving presets.
///
/// # Behavior
/// - Passed to `eframe::run_native` and rendered by the `update` method every frame.
/// - Handles preset selection, folder picking, validation, and final confirmation.
/// - Owns no internal lifetimes—uses shared mutable references for outward-facing state.
///
/// # Notes
/// - GUI layout and interactivity are driven entirely from the `update()` implementation.
/// - Not intended to be reused or retained beyond a single GUI session.
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
    open_manage_presets: bool,
    open_preset_index: Option<usize>,
    success_message: Option<(String, std::time::Instant)>,
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
        open_manage_presets: bool,
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
            open_manage_presets,
            open_preset_index: None,
            success_message: None,
        }
    }
}

impl eframe::App for ModeSelector<'_> {
    /// Handles rendering and user interaction for the GUI window each frame.
    ///
    /// # Parameters
    /// - `ctx`: The `egui::Context` used to render UI elements and issue commands.
    /// - `_frame`: The `eframe::Frame` which holds metadata about the current application window (not used directly).
    ///
    /// # Behavior
    /// - Builds a multi-section user interface inside a central panel.
    /// - Allows the user to:
    ///   - Select a target directory via a native folder picker.
    ///   - Choose a file type mode (e.g., Rust, JSON).
    ///   - Enable clipboard copy and recursive search options.
    ///   - Input folder names to ignore during recursion.
    ///   - Input additional command text.
    ///   - Select or manage command presets.
    ///   - Confirm their selections via an “OK” button.
    ///
    /// # UI Layout Summary
    /// 1. **Directory Picker Row**
    ///    - Button: “Select Directory” opens a folder picker.
    ///    - Displays the selected folder path in a read-only text box.
    /// 2. **File Type Mode Selection**
    ///    - Buttons for each mode with visual highlighting when selected.
    /// 3. **Options**
    ///    - Checkboxes for:
    ///      - Enabling clipboard copying.
    ///      - Enabling recursive directory search.
    ///    - If recursion is enabled:
    ///      - Multiline text box to enter ignored folders (one per line, case-insensitive).
    /// 4. **Additional Commands Input**
    ///    - Resizable multiline text area for arbitrary user instructions.
    /// 5. **Preset Commands**
    ///    - Dropdown for selecting one preset (with preview).
    ///    - Button to open preset manager for editing/adding/removing presets.
    /// 6. **Preset Manager Modal (if open)**
    ///    - Editable collapsible sections for each preset.
    ///    - Buttons to delete, add, and save presets.
    /// 7. **Warnings and Feedback**
    ///    - Displays a red warning if required selections are missing.
    ///    - Displays a green success message briefly after saving presets.
    /// 8. **Confirmation**
    ///    - “OK” button validates inputs and, if valid, closes the window with selections saved into shared state.
    ///
    /// # State Updates
    /// - Updates the following shared mutable state passed via the constructor:
    ///   - `selected_dir`
    ///   - `selected_mode`
    ///   - `preset_texts`
    ///   - `additional_commands`
    ///   - `enable_clipboard_copy`
    ///   - `enable_recursive_search`
    ///   - `ignored_folders`
    ///
    /// # Panics
    /// - This function does not panic.
    /// - All I/O and GUI logic is guarded and safely ignores failure (e.g., folder selection can fail silently).
    ///
    /// # Notes
    /// - The GUI closes via `ctx.send_viewport_cmd(egui::ViewportCommand::Close)` once all required inputs are provided.
    /// - Preset editing and saving are fully managed in the same GUI session without restarting.
    /// - State is preserved throughout the session but not across restarts (unless persisted in `presets.json`).
    ///
    /// # Example
    /// ```rust
    /// fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ///     egui::CentralPanel::default().show(ctx, |ui| {
    ///         ui.label("Hello GUI!");
    ///         // All layout and logic are handled here...
    ///     });
    /// }
    /// ```
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Directory Picker
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

            // Mode Selection
            ui.label("File Type Selection:");
            ui.horizontal_wrapped(|ui| {
                for mode in &self.modes {
                    let is_selected = self.selected_mode.as_deref() == Some(mode);
                    let (bg, fg) = if is_selected {
                        (egui::Color32::from_rgb(0, 120, 40), egui::Color32::WHITE)
                    } else {
                        (
                            ui.visuals().widgets.inactive.bg_fill,
                            ui.visuals().widgets.inactive.fg_stroke.color,
                        )
                    };

                    let btn =
                        ui.add(egui::Button::new(egui::RichText::new(mode).color(fg)).fill(bg));
                    if btn.clicked() {
                        *self.selected_mode = Some(mode.clone());
                        self.warning_message.clear();
                    }
                }
            });

            ui.checkbox(
                self.enable_clipboard_copy,
                "Enable save to clipboard automatically",
            );
            ui.checkbox(
                self.enable_recursive_search,
                "Enable recursive directory search",
            );

            if *self.enable_recursive_search {
                ui.group(|ui| {
                    ui.label("Ignore Folders (one per line, case insensitive):");
                    egui::ScrollArea::vertical()
                        .id_salt("ignored_folders_scroll")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(self.ignored_folders)
                                    .desired_width(ui.available_width())
                                    .desired_rows(4),
                            );
                        });
                });
            }

            // Additional commands
            ui.group(|ui| {
                ui.label("Additional Commands:");
                egui::ScrollArea::both()
                    .max_height(200.0)
                    .id_salt("Additional_commands_scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(self.additional_commands)
                                .desired_width(ui.available_width())
                                .desired_rows(5),
                        );
                    });
            });

            // Preset Command Dropdown
            ui.horizontal(|ui| {
                let selected_name_owned = self
                    .selected_presets
                    .iter()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "None".to_string());

                let preset_names: Vec<&str> =
                    self.presets.iter().map(|p| p.name.as_str()).collect();
                egui::ComboBox::from_label("Preset Command")
                    .selected_text(&selected_name_owned)
                    .show_ui(ui, |ui| {
                        for name in &preset_names {
                            if ui
                                .selectable_label(selected_name_owned == *name, *name)
                                .clicked()
                            {
                                self.selected_presets.clear();
                                self.selected_presets.insert(name.to_string());
                            }
                        }
                    });

                if ui.button("Manage Presets").clicked() {
                    self.open_manage_presets = true;
                }
            });

            if let Some(name) = self.selected_presets.iter().next() {
                if let Some(preset) = self.presets.iter().find(|p| p.name == *name) {
                    ui.label("Preview:");
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut preset.text.clone())
                                    .desired_width(ui.available_width())
                                    .desired_rows(4)
                                    .interactive(false),
                            );
                        });
                }
            }

            // Warning Message
            if !self.warning_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.warning_message);
            }

            // OK Button
            if ui.button("OK").clicked() {
                if self.selected_dir.is_none() {
                    self.warning_message = "⚠️ Please select a directory before proceeding!".into();
                } else if self.selected_mode.is_none() {
                    self.warning_message = "⚠️ Please select a file type before proceeding!".into();
                } else {
                    // Push selected preset text
                    for preset in &self.presets {
                        if self.selected_presets.contains(&preset.name) {
                            self.preset_texts.push(preset.text.clone());
                        }
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            // Preset Manager Window
            if self.open_manage_presets {
                if let Some((message, timestamp)) = &self.success_message {
                    if timestamp.elapsed().as_secs_f32() < 3.0 {
                        ctx.request_repaint_after(std::time::Duration::from_millis(100));
                        egui::Window::new("✔ Presets Saved")
                            .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
                            .resizable(false)
                            .collapsible(false)
                            .show(ctx, |ui| {
                                ui.label(
                                    egui::RichText::new(message).color(egui::Color32::LIGHT_GREEN),
                                );
                            });
                    } else {
                        self.success_message = None;
                    }
                }
                egui::Window::new("Preset Manager")
                    .collapsible(false)
                    .open(&mut self.open_manage_presets)
                    .show(ctx, |ui| {
                        let mut to_delete: Option<usize> = None;

                        for (i, preset) in self.presets.iter_mut().enumerate() {
                            let is_open = self.open_preset_index == Some(i);
                            let header_label = &preset.name;

                            let response = egui::CollapsingHeader::new(header_label)
                                .id_salt(format!("preset_{}", i))
                                .default_open(is_open)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        ui.text_edit_singleline(&mut preset.name);
                                        if ui.button("Delete").clicked() {
                                            to_delete = Some(i);
                                        }
                                    });

                                    ui.label("Text:");
                                    egui::ScrollArea::vertical()
                                        .max_height(100.0)
                                        .show(ui, |ui| {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut preset.text)
                                                    .desired_width(ui.available_width())
                                                    .desired_rows(4),
                                            );
                                        });

                                    ui.separator();
                                });

                            // Manual toggle behavior for stable collapse tracking
                            if response.header_response.clicked() {
                                if is_open {
                                    self.open_preset_index = None;
                                } else {
                                    self.open_preset_index = Some(i);
                                }
                            }
                        }

                        if let Some(i) = to_delete {
                            self.presets.remove(i);
                            if self.open_preset_index == Some(i) {
                                self.open_preset_index = None;
                            }
                        }

                        if ui.button("Add New Preset").clicked() {
                            self.presets.push(PresetCommand {
                                name: "New Preset".to_string(),
                                text: String::new(),
                            });
                        }

                        if ui.button("Save Changes").clicked() {
                            save_presets(&self.presets);
                            self.success_message = Some((
                                "✅ Presets saved successfully.".into(),
                                std::time::Instant::now(),
                            ));
                        }
                    });
            }

            if let Some((message, timestamp)) = &self.success_message {
                if timestamp.elapsed().as_secs_f32() < 3.0 {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                    ui.label(egui::RichText::new(message).color(egui::Color32::LIGHT_GREEN));
                } else {
                    self.success_message = None;
                }
            }
        });
    }
}
