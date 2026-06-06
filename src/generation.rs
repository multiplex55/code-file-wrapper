//! Shared output generation orchestration.
//!
//! This module is the single generation path used by both GUI and CLI entry points. Each
//! caller translates user input into a [`TagGenerationRequest`] and then calls
//! [`generate_tag_output`] to scan files, write tagged output, append preset/manual text,
//! and optionally copy the result to the clipboard.
//!
//! # Architecture Notes
//! - GUI and CLI flows should build requests instead of generating output independently.
//! - [`generate_tag_output`] validates generation-level inputs such as the root directory and output path.
//! - `file_ops.rs` remains limited to scanning and writing files; it does not own defaults or UI/CLI behavior.
//! - Output paths are caller-selected: both current entry points default to `tags_output.txt`, but both can override it.
//! - Callers remain responsible for presenting dialogs, printing summaries, or opening generated files.

use crate::file_ops::{append_additional_commands, write_folder_tags};
use crate::utils::copy_to_clipboard;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

/// Request data needed to generate a tagged output file.
#[derive(Debug, Clone)]
pub struct TagGenerationRequest {
    pub root_dir: PathBuf,
    pub extensions: Vec<String>,
    pub recursive: bool,
    pub ignored_folders: Vec<String>,
    pub output_path: PathBuf,
    pub additional_commands: String,
    pub preset_texts: Vec<String>,
    pub copy_to_clipboard: bool,
    pub open_after: bool,
}

/// Summary returned after tagged output generation completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationSummary {
    pub output_path: PathBuf,
    pub files_written: usize,
    pub files_skipped: usize,
    pub skipped_non_utf8_files: usize,
    pub recursive: bool,
}

/// Generates tagged output for a request without displaying GUI dialogs.
pub fn generate_tag_output(request: TagGenerationRequest) -> std::io::Result<GenerationSummary> {
    if !request.root_dir.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "root_dir must point to an existing directory",
        ));
    }

    if request.output_path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "output_path must not point to an existing directory",
        ));
    }

    let write_summary = write_folder_tags(
        &request.root_dir,
        &request.extensions,
        request.recursive,
        &request.ignored_folders,
        &request.output_path,
    )?;

    let combined_additional = combine_additional_commands(
        request.preset_texts.iter().map(String::as_str),
        &request.additional_commands,
    );

    if !combined_additional.trim().is_empty() {
        let output_path_string = request.output_path.to_string_lossy();
        append_additional_commands(&output_path_string, &combined_additional)?;
    }

    if request.copy_to_clipboard {
        let output_path_string = request.output_path.to_string_lossy();
        copy_to_clipboard(&output_path_string)?;
    }

    let _ = request.open_after;

    Ok(GenerationSummary {
        output_path: request.output_path,
        files_written: write_summary.files_written,
        files_skipped: write_summary.files_skipped,
        skipped_non_utf8_files: write_summary.skipped_non_utf8_files,
        recursive: request.recursive,
    })
}

fn combine_additional_commands<'a>(
    preset_texts: impl IntoIterator<Item = &'a str>,
    additional_commands: &str,
) -> String {
    let mut combined_additional = String::new();

    for preset_text in preset_texts {
        combined_additional.push('\n');
        combined_additional.push_str(preset_text.trim());
        combined_additional.push('\n');
    }

    if !additional_commands.trim().is_empty() {
        combined_additional.push('\n');
        combined_additional.push_str(additional_commands.trim());
        combined_additional.push('\n');
    }

    combined_additional
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn request(root_dir: PathBuf, output_path: PathBuf) -> TagGenerationRequest {
        TagGenerationRequest {
            root_dir,
            extensions: vec!["rs".to_string()],
            recursive: false,
            ignored_folders: Vec::new(),
            output_path,
            additional_commands: String::new(),
            preset_texts: Vec::new(),
            copy_to_clipboard: false,
            open_after: false,
        }
    }

    #[test]
    fn generate_tag_output_uses_custom_output_path_without_tags_output_txt() -> std::io::Result<()>
    {
        let temp = tempdir()?;
        let project = temp.path().join("project");
        fs::create_dir_all(project.join("src"))?;
        fs::write(project.join("src").join("main.rs"), "fn main() {}")?;
        let output_path = temp.path().join("project_context.txt");
        let mut request = request(project.clone(), output_path.clone());
        request.recursive = true;

        let summary = generate_tag_output(request)?;

        assert_eq!(summary.output_path, output_path);
        assert_eq!(summary.files_written, 1);
        assert_eq!(summary.files_skipped, 0);
        assert!(output_path.exists());
        assert!(!temp.path().join("tags_output.txt").exists());
        assert!(!project.join("tags_output.txt").exists());
        let output = fs::read_to_string(output_path)?;
        assert!(output.contains("main.rs"));
        assert!(output.contains("fn main() {}"));

        Ok(())
    }

    #[test]
    fn generate_tag_output_honors_recursive_and_ignore_options() -> std::io::Result<()> {
        let temp = tempdir()?;
        let project = temp.path().join("project");
        fs::create_dir_all(project.join("src"))?;
        fs::write(project.join("src").join("main.rs"), "fn main() {}")?;
        fs::write(project.join("src").join("lib.rs"), "pub fn lib() {}")?;
        fs::create_dir_all(project.join("target"))?;
        fs::write(
            project.join("target").join("generated.rs"),
            "pub fn generated() {}",
        )?;
        let output_path = temp.path().join("context.txt");
        let mut request = request(project, output_path.clone());
        request.recursive = true;
        request.ignored_folders = vec!["target".to_string()];

        let summary = generate_tag_output(request)?;

        assert_eq!(summary.files_written, 2);
        assert!(summary.recursive);
        let output = fs::read_to_string(output_path)?;
        assert!(output.contains("main.rs"));
        assert!(output.contains("lib.rs"));
        assert!(!output.contains("generated.rs"));
        assert!(!output.contains("pub fn generated() {}"));

        Ok(())
    }

    #[test]
    fn preset_text_appears_before_additional_commands() -> std::io::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().to_path_buf();
        fs::write(root.join("lib.rs"), "pub fn lib() {}")?;
        let output_path = root.join("ordered-tags.txt");
        let mut request = request(root, output_path.clone());
        request.preset_texts = vec!["preset instructions".to_string()];
        request.additional_commands = "manual instructions".to_string();

        generate_tag_output(request)?;

        let output = fs::read_to_string(output_path)?;
        let preset_index = output
            .find("preset instructions")
            .expect("preset text exists");
        let manual_index = output
            .find("manual instructions")
            .expect("manual command text exists");
        assert!(preset_index < manual_index);

        Ok(())
    }

    #[test]
    fn empty_additional_and_preset_text_does_not_add_additional_commands_block(
    ) -> std::io::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().to_path_buf();
        fs::write(root.join("lib.rs"), "pub fn lib() {}")?;
        let output_path = root.join("no-extra-block.txt");
        let mut request = request(root, output_path.clone());
        request.preset_texts = vec!["   ".to_string(), "\n".to_string()];
        request.additional_commands = " \n\t ".to_string();

        generate_tag_output(request)?;

        let output = fs::read_to_string(output_path)?;
        assert_eq!(output.matches("[Additional Commands]").count(), 1);

        Ok(())
    }

    #[test]
    fn invalid_root_directory_returns_error() -> std::io::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().join("missing");
        let output_path = temp.path().join("output.txt");

        let error = generate_tag_output(request(root, output_path)).expect_err("expected error");

        assert_eq!(error.kind(), ErrorKind::InvalidInput);

        Ok(())
    }

    #[test]
    fn output_path_pointing_at_existing_directory_returns_error() -> std::io::Result<()> {
        let temp = tempdir()?;
        let root = temp.path().to_path_buf();
        let output_dir = root.join("output-dir");
        fs::create_dir(&output_dir)?;

        let error = generate_tag_output(request(root, output_dir)).expect_err("expected error");

        assert_eq!(error.kind(), ErrorKind::InvalidInput);

        Ok(())
    }
}
