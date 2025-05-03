use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

const PRESET_FILE: &str = "presets.json";

/// # Presets Module
///
/// This module defines preset commands that can be selected in the GUI.
// Each preset consists of a name and corresponding text.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetCommand {
    pub name: String,
    pub text: String,
}

/// Returns a list of preset commands available in the application.
///
/// # Returns
/// - A `Vec<PresetCommand>` containing predefined command presets.
///   Each `PresetCommand` consists of a `name` (a short description) and `text` (detailed command content).
///
/// # Behavior
/// - This function constructs and returns a vector of `PresetCommand` instances.
/// - Each preset represents a predefined user command that can be selected in the GUI.
///
/// # Preset Structure
/// Each preset consists of:
/// - `name`: A short descriptive label displayed in the GUI.
/// - `text`: The actual content that will be inserted when the preset is selected.
///
/// # Example Usage
/// ```rust
/// let presets = get_presets();
/// for preset in presets {
///     println!("Preset: {} -> {}", preset.name, preset.text);
/// }
/// ```
///
/// # Available Presets
/// - **"Create Function Documentation"**: Generates structured documentation for functions.
/// - **"Create Readme"**: Generates an extensive README file with rich formatting.
/// - **"Button 3" - "Button 5"**: Reserved for future extensions.
///
/// # Notes
/// - The function returns static string slices (`&'static str`) for better memory efficiency.
/// - The content of `text` can be multiline, allowing for detailed instructions.
/// - The function is useful for populating UI elements or handling predefined automation tasks.
pub fn get_presets() -> Vec<PresetCommand> {
    vec![
        PresetCommand {
            name: "Create Function Documentation".to_string(),
            text: r#"for each function, create very detailed documentation for that function, only respond with a single function and even still only respond with the documentation  and not the contents of the function itself. After providing the documentation, prompt for the next function.

provide information such as panics, parameters, all the interesting stuff about that function

write the documentation as code blocks that appear above the function, similar to how other proper documentated rust functions are with code blocks

prompt with the name of the function, do not respond with the code body of the function, only the documentation
start with the main function

use "///" for the function blocks"#.to_string(),
        },
        PresetCommand {
            name: "Create Readme".to_string(),
            text: r#"I want you to update the readme.md file. I want this readme to be the most fancy readme possible with as many fancy emojis, information, examples, and other interesting things.
any sort of flowcharts, sequence diagrams, or other things that would look really good on a public facing github page are welcome
"#.to_string(),
        },
        PresetCommand { name: "Button 3".to_string(), text: "tbd".to_string() },
        PresetCommand { name: "Button 4".to_string(), text: "tbd".to_string() },
        PresetCommand { name: "Button 5".to_string(), text: "tbd".to_string() },
    ]
}

pub fn load_presets() -> Vec<PresetCommand> {
    if Path::new(PRESET_FILE).exists() {
        let data = fs::read_to_string(PRESET_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![PresetCommand {
            name: "Create Function Documentation".to_string(),
            text: "...".to_string(),
        }]
    }
}

pub fn save_presets(presets: &[PresetCommand]) {
    let data = serde_json::to_string_pretty(presets).unwrap_or_default();
    let _ = fs::write(PRESET_FILE, data);
}
