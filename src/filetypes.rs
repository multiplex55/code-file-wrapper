use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILETYPES_FILE: &str = "filetypes.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeGroup {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Finds a file type group by name using case-insensitive exact matching.
///
/// This helper intentionally does not perform fuzzy or partial matching so CLI
/// callers can resolve explicit presets without surprising ambiguity.
pub fn find_filetype_group<'a>(
    groups: &'a [FileTypeGroup],
    name: &str,
) -> Option<&'a FileTypeGroup> {
    groups
        .iter()
        .find(|group| group.name.eq_ignore_ascii_case(name))
}

/// Formats available file type group names for user-facing error messages.
///
/// Each group name is rendered on its own line as `- Name`.
pub fn format_available_filetype_groups(groups: &[FileTypeGroup]) -> String {
    groups
        .iter()
        .map(|group| format!("- {}", group.name))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Loads file type groups from `filetypes.json` or initializes with a default set if the file is missing.
///
/// # Purpose
/// Enables dynamic extension filtering by loading user-defined file type groups (e.g., Rust, JSON, Lua).
/// Falls back to a predefined set and persists it to disk if the config file does not yet exist.
///
/// # Behavior
/// - Checks for the existence of `filetypes.json` in the current working directory.
/// - If the file exists:
///   - Attempts to read and deserialize its contents into a `Vec<FileTypeGroup>`.
///   - Returns an empty list if deserialization fails (malformed or incompatible structure).
/// - If the file does not exist:
///   - Constructs a default list of file type groups:
///     - "Rust": `["rs"]`
///     - "JSON": `["json"]`
///     - "Lua": `["lua"]`
///   - Saves this default list immediately to `filetypes.json`.
///   - Returns the default list.
///
/// # Returns
/// A list of `FileTypeGroup` instances representing user-configured or default file type groups.
///
/// # Side Effects
/// - May create or overwrite `filetypes.json` with default values if it does not exist.
///
/// # Panics
/// - This function does not panic.
/// - Errors during file read/deserialize result in graceful fallback to empty or default state.
///
/// # File Format
/// ```json
/// [
///   { "name": "Rust", "extensions": ["rs"] },
///   { "name": "JSON", "extensions": ["json"] },
///   { "name": "Lua", "extensions": ["lua"] }
/// ]
/// ```
///
/// # Example
/// ```rust
/// let groups = get_filetypes();
/// for group in groups {
///     println!("{}: {:?}", group.name, group.extensions);
/// }
/// ```
///
/// # See Also
/// - [`save_filetypes`]: Used internally to persist defaults if the file is missing.
/// - [`FileTypeGroup`]: The core struct representing each group.
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

/// Saves the current list of file type groups to `filetypes.json` in a human-readable format.
///
/// # Purpose
/// Persists user-defined or default file extension groups so they can be reloaded in future sessions.
/// This allows customization of which file types the application should process (e.g., Rust, Lua, JSON).
///
/// # Parameters
/// - `groups`: A slice of `FileTypeGroup` instances representing each group of related extensions to save.
///   - Each group includes:
///     - `name`: A descriptive label (e.g., `"Rust"`, `"Web"`).
///     - `extensions`: A list of associated file extensions, e.g., `["rs", "ron"]`.
///
/// # Behavior
/// - Serializes the list into pretty-printed JSON using `serde_json`.
/// - Writes the result to a file named `filetypes.json` in the current working directory.
/// - If serialization fails, writes an empty string to avoid crashing.
///
/// # File Format
/// ```json
/// [
///   {
///     "name": "Rust",
///     "extensions": ["rs"]
///   },
///   {
///     "name": "JSON",
///     "extensions": ["json"]
///   }
/// ]
/// ```
///
/// # Side Effects
/// - Overwrites `filetypes.json` completely with the new contents.
/// - Does not perform deduplication or validation of extensions — this is the caller’s responsibility.
///
/// # Panics
/// - This function does not panic.
/// - All I/O and serialization errors are handled gracefully; write errors are ignored silently.
///
/// # Limitations
/// - The output is UTF-8 encoded, but non-string extension types or invalid Unicode values will cause serialization failure.
/// - No error reporting mechanism is used — failure to save is silent.
///
/// # Example
/// ```rust
/// let groups = vec![
///     FileTypeGroup {
///         name: "Web".to_string(),
///         extensions: vec!["html".to_string(), "css".to_string(), "js".to_string()],
///     }
/// ];
/// save_filetypes(&groups);
/// ```
pub fn save_filetypes(groups: &[FileTypeGroup]) {
    let data = serde_json::to_string_pretty(groups).unwrap_or_default();
    let _ = fs::write(FILETYPES_FILE, data);
}

#[cfg(test)]
mod tests {
    use super::{find_filetype_group, format_available_filetype_groups, FileTypeGroup};

    fn group(name: &str) -> FileTypeGroup {
        FileTypeGroup {
            name: name.to_string(),
            extensions: vec![name.to_lowercase()],
        }
    }

    #[test]
    fn find_filetype_group_matches_exact_name_case_insensitively() {
        let groups = vec![group("Rust")];

        assert_eq!(
            find_filetype_group(&groups, "Rust").map(|g| g.name.as_str()),
            Some("Rust")
        );
        assert_eq!(
            find_filetype_group(&groups, "rust").map(|g| g.name.as_str()),
            Some("Rust")
        );
        assert_eq!(
            find_filetype_group(&groups, "RUST").map(|g| g.name.as_str()),
            Some("Rust")
        );
    }

    #[test]
    fn find_filetype_group_returns_none_for_unknown_name() {
        let groups = vec![group("Rust")];

        assert!(find_filetype_group(&groups, "Go").is_none());
    }

    #[test]
    fn find_filetype_group_does_not_match_partial_names() {
        let groups = vec![group("Rust")];

        assert!(find_filetype_group(&groups, "Ru").is_none());
    }

    #[test]
    fn find_filetype_group_does_not_select_prefix_group() {
        let groups = vec![group("Ruby"), group("Rust")];

        assert_eq!(
            find_filetype_group(&groups, "Rust").map(|g| g.name.as_str()),
            Some("Rust")
        );
        assert!(find_filetype_group(&groups, "Ru").is_none());
    }

    #[test]
    fn format_available_filetype_groups_lists_each_name_on_own_line() {
        let groups = vec![group("Rust"), group("JSON"), group("Lua")];

        assert_eq!(
            format_available_filetype_groups(&groups),
            "- Rust\n- JSON\n- Lua"
        );
    }
}
