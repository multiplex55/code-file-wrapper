//! # Presets Module
//!
//! Manages user-defined command presets that can be selected and inserted via the GUI.
//!
//! # Purpose
//! - Stores reusable instruction blocks or command templates.
//! - Allows users to quickly insert structured guidance into the generated output.
//! - Supports editing, saving, and restoring presets across sessions.
//!
//! # Key Components
//! - [`PresetCommand`]: The core data structure representing each preset.
//! - [`get_presets`]: Loads presets from `presets.json` or returns built-in defaults.
//! - [`save_presets`]: Saves the current list of presets to disk.
//!
//! # File Behavior
//! - Presets are saved as JSON in a file called `presets.json`.
//! - If the file is missing or corrupted, a default set is created and stored automatically.
//!
//! # Format
//! Each preset is stored as:
//! ```json
//! { "name": "Create Readme", "text": "..." }
//! ```
//!
//! # Notes
//! - Preset names are not required to be unique, but duplicates may confuse the UI.
//! - Presets can be managed entirely through the GUI (added, deleted, renamed).
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

const PRESET_FILE: &str = "presets.json";

/// Represents a user-defined command preset used within the application.
///
/// # Fields
/// - `name`: A short, descriptive label for the preset (e.g., `"Create Readme"`).
///   - Displayed in the GUI dropdown and preset manager.
///   - Must be unique if deduplication is desired (not enforced).
///
/// - `text`: The actual command or instruction content associated with the preset.
///   - Can span multiple lines.
///   - Inserted into the output file and/or used in clipboard content.
///
/// # Derives
/// - `Debug`: For printing and logging.
/// - `Clone`: Enables duplication, necessary for UI operations and internal mutation.
/// - `Serialize` / `Deserialize`: Enables reading from and writing to `presets.json` via `serde`.
///
/// # Usage
/// - Used throughout the preset manager GUI for viewing, editing, and saving presets.
/// - Stored in a persistent JSON file (`presets.json`) and reloaded on startup via `get_presets()`.
///
/// # Example
/// ```rust
/// let preset = PresetCommand {
///     name: "Add License Header".to_string(),
///     text: "Insert a license header at the top of each source file.".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetCommand {
    pub name: String,
    pub text: String,
}

/// Loads preset command definitions from `presets.json` or creates and returns a default set if the file is missing.
///
/// # Returns
/// - `Vec<PresetCommand>`: A vector of preset commands, where each preset contains:
///   - `name`: A short descriptive label (e.g., `"Create Function Documentation"`).
///   - `text`: A detailed command or instruction associated with that preset.
///
/// # Behavior
/// - Checks if the `presets.json` file exists in the current working directory.
/// - If it exists:
///   - Reads and deserializes its contents using `serde_json`.
///   - Returns the parsed presets, or an empty list if deserialization fails.
/// - If it does not exist:
///   - Constructs a default set of hardcoded presets (currently 5).
///   - Immediately saves them to `presets.json` using `save_presets()`.
///   - Returns the default list.
///
/// # Panics
/// - This function does **not** panic explicitly.
/// - All I/O and deserialization errors are gracefully handled and default to fallback behavior.
///
/// # File Format
/// The `presets.json` file should contain an array of serialized `PresetCommand` objects like:
/// ```json
/// [
///   { "name": "Create Readme", "text": "I want you to update the readme.md file..." }
/// ]
/// ```
///
/// # Notes
/// - If the file exists but contains invalid JSON, the function will silently fall back to an empty vector.
/// - Presets are user-editable outside of the app (e.g., manually modifying `presets.json`).
/// - Default presets include commands like:
///   - `"Create Function Documentation"`
///   - `"Create Readme"`
///   - `"Button 3"` through `"Button 5"` (placeholders).
///
/// # Example
/// ```rust
/// let presets = get_presets();
/// for preset in presets {
///     println!("{} => {}", preset.name, preset.text);
/// }
/// ```
pub fn get_presets() -> Vec<PresetCommand> {
    if Path::new(PRESET_FILE).exists() {
        let data = fs::read_to_string(PRESET_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let default_presets = vec![
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
        ];

        // Save them immediately
        save_presets(&default_presets);
        default_presets
    }
}

/// Saves a list of preset commands to `presets.json` in the current working directory.
///
/// # Parameters
/// - `presets`: A slice of `PresetCommand` structs representing the presets to be saved.
///   - Each preset includes:
///     - `name`: A user-visible label.
///     - `text`: The command or instruction content.
///
/// # Behavior
/// - Serializes the provided `presets` list into pretty-formatted JSON using `serde_json`.
/// - Writes the result to a file named `presets.json`, overwriting any existing contents.
/// - If serialization fails, an empty string is written (as a fallback, silently).
///
/// # Panics
/// - This function does not explicitly panic.
/// - Failures during serialization or writing are silently ignored (errors are not bubbled or logged).
///
/// # File Format
/// ```json
/// [
///   {
///     "name": "Create Function Documentation",
///     "text": "for each function, create very detailed documentation..."
///   },
///   {
///     "name": "Create Readme",
///     "text": "I want you to update the readme.md file..."
///   }
/// ]
/// ```
///
/// # Notes
/// - The output file is UTF-8 encoded and human-readable.
/// - No backup or deduplication is performed — this function **completely overwrites** the file.
/// - Used by the preset manager UI when saving changes or creating new presets.
///
/// # Example
/// ```rust
/// let presets = vec![
///     PresetCommand { name: "Test".into(), text: "echo Hello".into() }
/// ];
/// save_presets(&presets);
/// ```
pub fn save_presets(presets: &[PresetCommand]) {
    let data = serde_json::to_string_pretty(presets).unwrap_or_default();
    let _ = fs::write(PRESET_FILE, data);
}
