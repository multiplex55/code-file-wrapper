use crate::presets::{load_presets, save_presets};

/// # GUI Module
///
/// This module manages the graphical user interface (GUI) using `egui`.
/// It allows users to select a directory, file type, and preset commands.
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
    open_manage_presets: bool,

    open_preset_index: Option<usize>,
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
            presets: load_presets(),
            enable_recursive_search,
            ignored_folders,
            open_manage_presets,
            open_preset_index: None,
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
                egui::ScrollArea::both().max_height(200.0).show(ui, |ui| {
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
                    ui.label(&preset.text);
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
                egui::Window::new("Preset Manager")
                    .collapsible(false)
                    .open(&mut self.open_manage_presets)
                    .show(ctx, |ui| {
                        let mut to_delete: Option<usize> = None;

                        for (i, preset) in self.presets.iter_mut().enumerate() {
                            let is_open = self.open_preset_index == Some(i);
                            let header_label = format!("▶ {}", preset.name);

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
                        }
                    });
            }
        });
    }
}
