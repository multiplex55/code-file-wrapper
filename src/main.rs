mod file_ops;
use crate::file_ops::write_folder_tags;
use clipboard_win::{formats, Clipboard, Setter};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{read_to_string, OpenOptions};
use std::io;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let modes: HashMap<&str, Vec<&str>> = HashMap::from([
        ("AHK", vec!["ahk"]),
        ("Rust", vec!["rs"]),
        ("JSON", vec!["json"]),
        ("XML", vec!["xml"]),
        ("C/CPP", vec!["c", "cpp", "h"]),
    ]);

    let cursor_position = get_cursor_position();
    let (selected_dir, selected_mode, enable_clipboard_copy, additional_commands, preset_texts) =
        mode_selection_gui(modes.keys().cloned().collect(), cursor_position);

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

struct Preset {
    name: &'static str,
    text: &'static str,
}

fn mode_selection_gui(
    modes: Vec<&str>,
    initial_pos: Option<(f32, f32)>,
) -> (Option<PathBuf>, Option<String>, bool, String, Vec<String>) {
    let mut selected_mode: Option<String> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();
    let mut selected_dir: Option<PathBuf> = None;
    let mut preset_texts = Vec::new();

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
            .with_min_inner_size([500.0, 350.0]),
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

struct ModeSelector<'a> {
    modes: Vec<String>,
    selected_mode: &'a mut Option<String>,
    enable_clipboard_copy: &'a mut bool,
    additional_commands: &'a mut String,
    selected_dir: &'a mut Option<PathBuf>,
    preset_texts: &'a mut Vec<String>,
    selected_presets: HashSet<String>, // Track selected presets
    warning_message: String,
}

impl<'a> ModeSelector<'a> {
    fn new(
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
            selected_presets: HashSet::new(), // Initialize empty set
            warning_message: String::new(),
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
                        self.warning_message.clear(); // Clear warning on selection
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

            // Additional Commands Box
            ui.label("Additional Commands:");
            ui.add(
                egui::TextEdit::multiline(self.additional_commands)
                    .desired_width(400.0)
                    .desired_rows(5)
                    .clip_text(false),
            );

            // Preset Commands Selection
            ui.label("Preset Commands:");
            ui.horizontal_wrapped(|ui| {
                let presets = vec![
                    ("Button 1", "Preset text for Button 1"),
                    ("Button 2", "Preset text for Button 2"),
                    ("Button 3", "Preset text for Button 3"),
                    ("Button 4", "Preset text for Button 4"),
                    ("Button 5", "Preset text for Button 5"),
                ];

                for (name, text) in &presets {
                    let is_selected = self.selected_presets.contains(*name);
                    let default_bg = ui.visuals().widgets.inactive.bg_fill;
                    let default_text = ui.visuals().widgets.inactive.fg_stroke.color;

                    let (bg_color, text_color) = if is_selected {
                        (egui::Color32::from_rgb(0, 120, 40), egui::Color32::WHITE)
                    } else {
                        (default_bg, default_text)
                    };

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(*name).color(text_color))
                                .fill(bg_color),
                        )
                        .clicked()
                    {
                        if is_selected {
                            self.selected_presets.remove(*name);
                        } else {
                            self.selected_presets.insert(name.to_string());
                        }
                    }
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
                    for preset in &self.selected_presets {
                        let preset_text = match preset.as_str() {
                            "Button 1" => "Preset text for Button 1",
                            "Button 2" => "Preset text for Button 2",
                            "Button 3" => "Preset text for Button 3",
                            "Button 4" => "Preset text for Button 4",
                            "Button 5" => "Preset text for Button 5",
                            _ => "",
                        };
                        self.preset_texts.push(preset_text.to_string());
                    }

                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
    }
}

fn append_additional_commands(file_path: &str, additional_commands: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    writeln!(file, "\n[Additional Commands]")?;
    writeln!(file, "{}\n", additional_commands)?;
    Ok(())
}

fn copy_to_clipboard(file_path: &str) -> io::Result<()> {
    let file_contents = read_to_string(file_path)?;

    let _clip = Clipboard::new_attempts(10)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Clipboard access failed"))?;

    formats::Unicode
        .write_clipboard(&file_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard contents"))?;

    Ok(())
}

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
