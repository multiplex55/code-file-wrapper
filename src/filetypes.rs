use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILETYPES_FILE: &str = "filetypes.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeGroup {
    pub name: String,
    pub extensions: Vec<String>,
}

pub fn get_filetypes() -> Vec<FileTypeGroup> {
    if Path::new(FILETYPES_FILE).exists() {
        let data = fs::read_to_string(FILETYPES_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let default = vec![
            FileTypeGroup {
                name: "Rust".into(),
                extensions: vec!["rs".into()],
            },
            FileTypeGroup {
                name: "JSON".into(),
                extensions: vec!["json".into()],
            },
            FileTypeGroup {
                name: "Lua".into(),
                extensions: vec!["lua".into()],
            },
        ];
        save_filetypes(&default);
        default
    }
}

pub fn save_filetypes(groups: &[FileTypeGroup]) {
    let data = serde_json::to_string_pretty(groups).unwrap_or_default();
    let _ = fs::write(FILETYPES_FILE, data);
}
