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
            presets: get_presets(), // Fetch preset commands dynamically
        }
    }
}

impl eframe::App for ModeSelector<'_> {
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

            // File Type Selection
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

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(mode).color(text_color))
                                .fill(bg_color),
                        )
                        .clicked()
                    {
                        *self.selected_mode = Some(mode.clone());
                        self.warning_message.clear();
                    }
                }
            });

            // Clipboard Save Option
            ui.checkbox(
                self.enable_clipboard_copy,
                "Enable save to clipboard automatically",
            );

            // Additional Commands
            ui.label("Additional Commands:");
            ui.add(
                egui::TextEdit::multiline(self.additional_commands)
                    .desired_width(400.0)
                    .desired_rows(5)
                    .clip_text(false),
            );

            // Preset Selection
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

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(preset.name).color(text_color))
                                .fill(bg_color),
                        )
                        .clicked()
                    {
                        if is_selected {
                            self.selected_presets.remove(preset.name);
                        } else {
                            self.selected_presets.insert(preset.name.to_string());
                        }
                    }
                }
            });

            ui.separator();

            // Display Warnings
            if !self.warning_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.warning_message);
            }

            // OK Button - Validate Selections
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
