use crate::filetypes::{find_filetype_group, format_available_filetype_groups, FileTypeGroup};
use crate::generation::TagGenerationRequest;
use crate::presets::PresetCommand;
use clap::{Args, Parser, Subcommand};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "code-file-wrapper")]
#[command(about = "Wrap selected project files in tagged context output")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch the graphical interface.
    Gui,
    /// Generate output from command-line arguments.
    Run(RunArgs),
    /// Generate output from a saved profile.
    RunProfile { name: String },
    /// Print saved run profiles.
    ListProfiles,
    /// Delete a saved run profile.
    DeleteProfile { name: String },
    /// Save a reusable run profile from command-line arguments.
    SaveProfile(SaveProfileArgs),
    /// Print available file type groups from filetypes.json.
    ListFileTypes,
}

#[derive(Debug, Args)]
pub struct SaveProfileArgs {
    pub name: String,
    #[arg(long)]
    pub force: bool,
    #[command(flatten)]
    pub run: RunArgs,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub dir: PathBuf,
    #[arg(long = "file-type")]
    pub file_type: Option<String>,
    #[arg(long = "ext")]
    pub extensions: Vec<String>,
    #[arg(long)]
    pub recursive: bool,
    #[arg(long = "ignore")]
    pub ignored_folders: Vec<String>,
    #[arg(long, default_value = "tags_output.txt")]
    pub output: PathBuf,
    #[arg(long)]
    pub copy: bool,
    #[arg(long)]
    pub open: bool,
    #[arg(long = "preset")]
    pub presets: Vec<String>,
    #[arg(long = "additional")]
    pub additional_commands: Option<String>,
    #[arg(long = "additional-file")]
    pub additional_commands_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BuiltRunRequest {
    pub request: TagGenerationRequest,
    pub extensions_used: Vec<String>,
}

pub fn build_run_request(
    args: RunArgs,
    file_type_groups: &[FileTypeGroup],
    presets: &[PresetCommand],
) -> Result<BuiltRunRequest, String> {
    validate_run_directory(&args.dir)?;
    let extensions = resolve_extensions(&args, file_type_groups)?;
    let preset_texts = resolve_presets(&args.presets, presets)?;
    let additional_commands = resolve_additional_commands(
        args.additional_commands_file.as_ref(),
        args.additional_commands.as_deref(),
    )?;

    Ok(BuiltRunRequest {
        extensions_used: extensions.clone(),
        request: TagGenerationRequest {
            root_dir: args.dir,
            extensions,
            recursive: args.recursive,
            ignored_folders: args.ignored_folders,
            output_path: args.output,
            additional_commands,
            preset_texts,
            copy_to_clipboard: args.copy,
            open_after: args.open,
        },
    })
}

fn validate_run_directory(dir: &PathBuf) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!(
            "Directory '{}' does not exist or is not a folder.",
            dir.display()
        ));
    }

    Ok(())
}

fn resolve_extensions(
    args: &RunArgs,
    file_type_groups: &[FileTypeGroup],
) -> Result<Vec<String>, String> {
    let mut extensions = Vec::new();

    if let Some(file_type) = args.file_type.as_deref() {
        let group = find_filetype_group(file_type_groups, file_type).ok_or_else(|| {
            format!(
                "Unknown file type group '{file_type}'. Available file type groups:\n{}",
                format_available_filetype_groups(file_type_groups)
            )
        })?;
        extensions.extend(
            group
                .extensions
                .iter()
                .map(|extension| normalize_extension(extension)),
        );
    }

    extensions.extend(
        args.extensions
            .iter()
            .map(|extension| normalize_extension(extension)),
    );

    extensions.retain(|extension| !extension.is_empty());
    deduplicate_preserving_order(&mut extensions);

    if extensions.is_empty() {
        return Err(
            "No extensions selected. Provide --file-type <group> or one or more --ext <extension> values."
                .to_string(),
        );
    }

    Ok(extensions)
}

fn normalize_extension(extension: &str) -> String {
    extension.trim().trim_start_matches('.').to_string()
}

fn deduplicate_preserving_order(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

fn resolve_presets(
    requested_presets: &[String],
    presets: &[PresetCommand],
) -> Result<Vec<String>, String> {
    requested_presets
        .iter()
        .map(|requested_name| {
            presets
                .iter()
                .find(|preset| preset.name.eq_ignore_ascii_case(requested_name))
                .map(|preset| preset.text.clone())
                .ok_or_else(|| {
                    format!(
                        "Unknown preset '{requested_name}'. Available presets:\n{}",
                        format_available_presets(presets)
                    )
                })
        })
        .collect()
}

fn format_available_presets(presets: &[PresetCommand]) -> String {
    presets
        .iter()
        .map(|preset| format!("- {}", preset.name))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn resolve_additional_commands(
    additional_commands_file: Option<&PathBuf>,
    additional_commands: Option<&str>,
) -> Result<String, String> {
    let file_content = additional_commands_file
        .map(|path| {
            fs::read_to_string(path).map_err(|error| {
                format!(
                    "Failed to read additional commands file '{}': {error}",
                    path.display()
                )
            })
        })
        .transpose()?;

    let mut parts = Vec::new();
    if let Some(file_content) = file_content {
        if !file_content.trim().is_empty() {
            parts.push(file_content);
        }
    }
    if let Some(additional_commands) = additional_commands {
        if !additional_commands.trim().is_empty() {
            parts.push(additional_commands.to_string());
        }
    }

    Ok(parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::tempdir;

    fn rust_group() -> Vec<FileTypeGroup> {
        vec![FileTypeGroup {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
        }]
    }

    #[test]
    fn run_with_file_type_parses_correctly() {
        let cli = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            ".",
            "--file-type",
            "Rust",
            "--recursive",
            "--output",
            "context.txt",
        ])
        .expect("CLI should parse");

        let Some(Command::Run(args)) = cli.command else {
            panic!("expected run command");
        };

        assert_eq!(args.dir, PathBuf::from("."));
        assert_eq!(args.file_type.as_deref(), Some("Rust"));
        assert!(args.recursive);
        assert_eq!(args.output, PathBuf::from("context.txt"));
    }

    #[test]
    fn repeated_ext_values_parse() {
        let cli = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            ".",
            "--ext",
            "rs",
            "--ext",
            "toml",
            "--ext",
            "md",
        ])
        .expect("CLI should parse");

        let Some(Command::Run(args)) = cli.command else {
            panic!("expected run command");
        };

        assert_eq!(args.extensions, vec!["rs", "toml", "md"]);
    }

    #[test]
    fn repeated_ignore_values_parse() {
        let cli = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            ".",
            "--ext",
            "rs",
            "--ignore",
            "target",
            "--ignore",
            ".git",
        ])
        .expect("CLI should parse");

        let Some(Command::Run(args)) = cli.command else {
            panic!("expected run command");
        };

        assert_eq!(args.ignored_folders, vec!["target", ".git"]);
    }

    #[test]
    fn no_arg_cli_has_no_command() {
        let cli = Cli::try_parse_from(["code-file-wrapper"]).expect("CLI should parse");

        assert!(cli.command.is_none());
    }

    #[test]
    fn build_run_request_combines_and_deduplicates_extensions() -> std::io::Result<()> {
        let temp = tempdir()?;
        let dir = temp.path().to_str().expect("temp path is UTF-8");
        let args = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            dir,
            "--file-type",
            "Rust",
            "--ext",
            ".toml",
            "--ext",
            "rs",
        ])
        .expect("CLI should parse");
        let Some(Command::Run(args)) = args.command else {
            panic!("expected run command");
        };

        let built = build_run_request(args, &rust_group(), &[]).expect("request should build");

        assert_eq!(built.extensions_used, vec!["rs", "toml"]);
        assert_eq!(built.request.extensions, vec!["rs", "toml"]);
        assert_eq!(built.request.root_dir, temp.path());
        Ok(())
    }

    #[test]
    fn build_run_request_requires_file_type_or_ext_before_generation() -> std::io::Result<()> {
        let temp = tempdir()?;
        let dir = temp.path().to_str().expect("temp path is UTF-8");
        let args = Cli::try_parse_from(["code-file-wrapper", "run", "--dir", dir])
            .expect("CLI should parse");
        let Some(Command::Run(args)) = args.command else {
            panic!("expected run command");
        };

        let error = build_run_request(args, &rust_group(), &[]).expect_err("expected error");

        assert!(error.contains("Provide --file-type"));
        assert!(error.contains("--ext"));
        Ok(())
    }

    #[test]
    fn unknown_file_type_lists_available_groups() -> std::io::Result<()> {
        let temp = tempdir()?;
        let dir = temp.path().to_str().expect("temp path is UTF-8");
        let args = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            dir,
            "--file-type",
            "Go",
        ])
        .expect("CLI should parse");
        let Some(Command::Run(args)) = args.command else {
            panic!("expected run command");
        };

        let error = build_run_request(args, &rust_group(), &[]).expect_err("expected error");

        assert!(error.contains("Unknown file type group 'Go'"));
        assert!(error.contains("Available file type groups"));
        assert!(error.contains("- Rust"));
        Ok(())
    }

    #[test]
    fn additional_file_content_precedes_inline_additional_commands() -> std::io::Result<()> {
        let temp = tempdir()?;
        let additional_path = temp.path().join("additional.txt");
        fs::write(&additional_path, "from file")?;
        let dir = temp.path().to_str().expect("temp path is UTF-8");
        let args = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            dir,
            "--ext",
            "rs",
            "--additional-file",
            additional_path.to_str().expect("path is UTF-8"),
            "--additional",
            "inline",
        ])
        .expect("CLI should parse");
        let Some(Command::Run(args)) = args.command else {
            panic!("expected run command");
        };

        let built = build_run_request(args, &rust_group(), &[]).expect("request should build");

        assert_eq!(built.request.additional_commands, "from file\ninline");
        Ok(())
    }

    #[test]
    fn unknown_preset_lists_available_presets() {
        let temp = tempdir().expect("tempdir should be created");
        let dir = temp.path().to_str().expect("temp path is UTF-8");
        let args = Cli::try_parse_from([
            "code-file-wrapper",
            "run",
            "--dir",
            dir,
            "--ext",
            "rs",
            "--preset",
            "missing",
        ])
        .expect("CLI should parse");
        let Some(Command::Run(args)) = args.command else {
            panic!("expected run command");
        };
        let presets = vec![PresetCommand {
            name: "Known".to_string(),
            text: "preset text".to_string(),
        }];

        let error = build_run_request(args, &rust_group(), &presets).expect_err("expected error");

        assert!(error.contains("Unknown preset 'missing'"));
        assert!(error.contains("- Known"));
    }
}
