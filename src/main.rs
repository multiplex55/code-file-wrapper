mod file_ops;
use crate::file_ops::write_folder_tags;
use clipboard_win::{formats, Clipboard, Setter};
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
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

fn mode_selection_gui(
    modes: Vec<&str>,
    initial_pos: Option<(f32, f32)>,
) -> (Option<PathBuf>, Option<String>, bool, String, Vec<String>) {
    let mut selected_mode: Option<String> = None;
    let mut enable_clipboard_copy = false;
    let mut additional_commands = String::new();
    let mut selected_dir: Option<PathBuf> = None;
    let mut preset_texts = Vec::new();
    let mut selected_file_type: Option<String> = None;

    let app = ModeSelector::new(
        modes,
        &mut selected_mode,
        &mut enable_clipboard_copy,
        &mut additional_commands,
        &mut selected_dir,
        &mut preset_texts,
        &mut selected_file_type,
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

struct Preset {
    name: &'static str,
    text: &'static str,
}

struct ModeSelector<'a> {
    modes: Vec<String>,
    selected_mode: &'a mut Option<String>,
    enable_clipboard_copy: &'a mut bool,
    additional_commands: &'a mut String,
    selected_dir: &'a mut Option<PathBuf>,
    preset_texts: &'a mut Vec<String>,
    selected_file_type: &'a mut Option<String>,
}

impl<'a> ModeSelector<'a> {
    fn new(
        modes: Vec<&str>,
        selected_mode: &'a mut Option<String>,
        enable_clipboard_copy: &'a mut bool,
        additional_commands: &'a mut String,
        selected_dir: &'a mut Option<PathBuf>,
        preset_texts: &'a mut Vec<String>,
        selected_file_type: &'a mut Option<String>,
    ) -> Self {
        Self {
            modes: modes.into_iter().map(String::from).collect(),
            selected_mode,
            enable_clipboard_copy,
            additional_commands,
            selected_dir,
            preset_texts,
            selected_file_type,
        }
    }
}

impl eframe::App for ModeSelector<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Select Directory").clicked() {
                    if let Some(dir) = FileDialog::new().set_directory(".").pick_folder() {
                        *self.selected_dir = Some(dir);
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
                    let is_selected = self.selected_file_type.as_deref() == Some(mode);
                    let color = if is_selected {
                        egui::Color32::from_rgb(4, 175, 80)
                    } else {
                        egui::Color32::default()
                    };

                    if ui.add(egui::Button::new(mode).fill(color)).clicked() {
                        *self.selected_file_type = Some(mode.clone());
                    }
                }
            });

            ui.checkbox(
                self.enable_clipboard_copy,
                "Enable save to clipboard automatically",
            );

            ui.label("Additional Commands:");
            ui.add(
                egui::TextEdit::multiline(self.additional_commands)
                    .desired_width(400.0)
                    .desired_rows(5)
                    .clip_text(false),
            );

            ui.label("Preset Commands:");
            ui.horizontal_wrapped(|ui| {
                let presets = vec![
                    Preset {
                        name: "Button 1",
                        text: "Preset text for Button 1",
                    },
                    Preset {
                        name: "Button 2",
                        text: "Preset text for Button 2",
                    },
                    Preset {
                        name: "Button 3",
                        text: "Preset text for Button 3",
                    },
                    Preset {
                        name: "Button 4",
                        text: "Preset text for Button 4",
                    },
                    Preset {
                        name: "Button 5",
                        text: "Preset text for Button 5",
                    },
                ];

                for preset in &presets {
                    if ui.button(preset.name).clicked() {
                        self.preset_texts.push(preset.text.to_string());
                    }
                }
            });
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
