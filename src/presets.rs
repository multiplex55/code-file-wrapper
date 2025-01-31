//! # Presets Module
//!
//! This module defines preset commands that can be selected in the GUI.
//! Each preset consists of a name and corresponding text.

#[derive(Debug, Clone)]
pub struct PresetCommand {
    pub name: &'static str,
    pub text: &'static str,
}

/// Returns a list of preset commands.
///
/// # Returns
/// - `Vec<PresetCommand>`: A list of available preset commands.
pub fn get_presets() -> Vec<PresetCommand> {
    vec![
        PresetCommand {
            name: "Button 1",
            text: "Preset text for Button 1",
        },
        PresetCommand {
            name: "Button 2",
            text: "Preset text for Button 2",
        },
        PresetCommand {
            name: "Button 3",
            text: "Preset text for Button 3",
        },
        PresetCommand {
            name: "Button 4",
            text: "Preset text for Button 4",
        },
        PresetCommand {
            name: "Button 5",
            text: "Preset text for Button 5",
        },
    ]
}
