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
            name: "Create Function Documentation",
            text: r#"for each function, create very detailed documentation for that function, only respond with a single function and even still only respond with the documentation  and not the contents of the function itself. After providing the documentation, prompt for the next function.
 
provide information such as panics, parameters, all the interesting stuff about that function

write the documentation as code blocks that appear above the function, similar to how other proper documentated rust functions are with code blocks

start with the main function

use "///" for the function blocks"#,
        },
        PresetCommand {
            name: "Create Readme",
            text: r#"I want you to update the readme.md file. I want this readme to be the most fancy readme possible with as many fancy emojis, information, examples, and other interesting things.
any sort of flowcharts, sequence diagrams, or other things that would look really good on a public facing github page are welcome
"#,
        },
        PresetCommand {
            name: "Button 3",
            text: "tbd",
        },
        PresetCommand {
            name: "Button 4",
            text: "tbd",
        },
        PresetCommand {
            name: "Button 5",
            text: "tbd",
        },
    ]
}
