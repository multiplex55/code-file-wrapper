use crate::cli::{build_run_request, BuiltRunRequest, RunArgs};
use crate::filetypes::FileTypeGroup;
use crate::presets::PresetCommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

pub const PROFILES_FILE: &str = "profiles.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProfile {
    pub name: String,
    pub dir: PathBuf,
    pub file_type: Option<String>,
    pub extensions: Vec<String>,
    pub recursive: bool,
    pub ignored_folders: Vec<String>,
    pub output: PathBuf,
    pub copy: bool,
    pub open: bool,
    pub presets: Vec<String>,
    pub additional_commands: Option<String>,
}

pub fn load_profiles() -> Vec<RunProfile> {
    load_profiles_from_path(PROFILES_FILE).unwrap_or_default()
}

pub fn save_profiles(profiles: &[RunProfile]) -> std::io::Result<()> {
    save_profiles_to_path(PROFILES_FILE, profiles)
}

pub fn find_profile<'a>(profiles: &'a [RunProfile], name: &str) -> Option<&'a RunProfile> {
    profiles.iter().find(|profile| profile.name == name)
}

pub fn load_profiles_from_path(path: impl AsRef<Path>) -> std::io::Result<Vec<RunProfile>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&data).map_err(|error| Error::new(ErrorKind::InvalidData, error))
}

pub fn save_profiles_to_path(
    path: impl AsRef<Path>,
    profiles: &[RunProfile],
) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(profiles)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    fs::write(path, data)
}

pub fn profile_from_run_args(name: String, args: RunArgs) -> Result<RunProfile, String> {
    if name.trim().is_empty() {
        return Err("Profile name is required.".to_string());
    }

    let additional_commands = crate::cli::resolve_additional_commands(
        args.additional_commands_file.as_ref(),
        args.additional_commands.as_deref(),
    )?;

    Ok(RunProfile {
        name,
        dir: args.dir,
        file_type: args.file_type,
        extensions: args.extensions,
        recursive: args.recursive,
        ignored_folders: args.ignored_folders,
        output: args.output,
        copy: args.copy,
        open: args.open,
        presets: args.presets,
        additional_commands: if additional_commands.trim().is_empty() {
            None
        } else {
            Some(additional_commands)
        },
    })
}

pub fn save_profile(profile: RunProfile, force: bool) -> Result<(), String> {
    let mut profiles = load_profiles();
    upsert_profile(&mut profiles, profile, force)?;
    save_profiles(&profiles).map_err(|error| format!("Failed to save profiles: {error}"))
}

pub fn upsert_profile(
    profiles: &mut Vec<RunProfile>,
    profile: RunProfile,
    force: bool,
) -> Result<(), String> {
    if profile.name.trim().is_empty() {
        return Err("Profile name is required.".to_string());
    }

    if let Some(existing_index) = profiles.iter().position(|p| p.name == profile.name) {
        if !force {
            return Err(format!(
                "Profile '{}' already exists. Re-run with --force to overwrite it.",
                profile.name
            ));
        }
        profiles[existing_index] = profile;
    } else {
        profiles.push(profile);
    }

    Ok(())
}

pub fn delete_profile(name: &str) -> Result<(), String> {
    let mut profiles = load_profiles();
    delete_profile_from_list(&mut profiles, name)?;
    save_profiles(&profiles).map_err(|error| format!("Failed to save profiles: {error}"))
}

pub fn delete_profile_from_list(profiles: &mut Vec<RunProfile>, name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Profile name is required.".to_string());
    }

    let initial_len = profiles.len();
    profiles.retain(|profile| profile.name != name);

    if profiles.len() == initial_len {
        return Err(format!("Profile '{name}' does not exist."));
    }

    Ok(())
}

pub fn profile_to_run_request(
    profile: &RunProfile,
    file_type_groups: &[FileTypeGroup],
    presets: &[PresetCommand],
) -> Result<BuiltRunRequest, String> {
    if !profile.dir.is_dir() {
        return Err(format!(
            "Profile '{}' directory '{}' does not exist or is not a folder.",
            profile.name,
            profile.dir.display()
        ));
    }

    build_run_request(profile.clone().into_run_args(), file_type_groups, presets)
        .map_err(|error| format!("Profile '{}': {error}", profile.name))
}

impl RunProfile {
    fn into_run_args(self) -> RunArgs {
        RunArgs {
            dir: self.dir,
            file_type: self.file_type,
            extensions: self.extensions,
            recursive: self.recursive,
            ignored_folders: self.ignored_folders,
            output: self.output,
            copy: self.copy,
            open: self.open,
            presets: self.presets,
            additional_commands: self.additional_commands,
            additional_commands_file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn profile(name: &str) -> RunProfile {
        RunProfile {
            name: name.to_string(),
            dir: PathBuf::from("."),
            file_type: Some("Rust".to_string()),
            extensions: vec!["toml".to_string()],
            recursive: true,
            ignored_folders: vec!["target".to_string()],
            output: PathBuf::from("context.txt"),
            copy: false,
            open: true,
            presets: vec!["Known".to_string()],
            additional_commands: Some("extra".to_string()),
        }
    }

    #[test]
    fn save_and_reload_profiles_from_temp_json_file() -> std::io::Result<()> {
        let temp = tempdir()?;
        let path = temp.path().join("profiles.json");
        let profiles = vec![profile("daily")];

        save_profiles_to_path(&path, &profiles)?;
        let reloaded = load_profiles_from_path(&path)?;

        assert_eq!(reloaded, profiles);
        Ok(())
    }

    #[test]
    fn find_profile_by_exact_name() {
        let profiles = vec![profile("daily"), profile("Daily")];

        let found = find_profile(&profiles, "daily").expect("profile should exist");

        assert_eq!(found.name, "daily");
        assert!(find_profile(&profiles, "DAILY").is_none());
    }

    #[test]
    fn reject_duplicate_save_without_force() {
        let mut profiles = vec![profile("daily")];
        let error = upsert_profile(&mut profiles, profile("daily"), false)
            .expect_err("duplicate should fail without force");

        assert!(error.contains("already exists"));
        assert!(error.contains("--force"));
    }

    #[test]
    fn delete_existing_profile() {
        let mut profiles = vec![profile("daily"), profile("weekly")];

        delete_profile_from_list(&mut profiles, "daily").expect("delete should succeed");

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "weekly");
    }

    #[test]
    fn deleting_missing_profile_returns_useful_error() {
        let mut profiles = vec![profile("daily")];

        let error = delete_profile_from_list(&mut profiles, "missing")
            .expect_err("missing profile should fail");

        assert!(error.contains("Profile 'missing' does not exist"));
    }

    #[test]
    fn running_profile_with_unknown_file_type_errors_before_generation() -> std::io::Result<()> {
        let temp = tempdir()?;
        let mut profile = profile("bad-type");
        profile.dir = temp.path().to_path_buf();
        profile.file_type = Some("Unknown".to_string());
        profile.extensions.clear();

        let error =
            profile_to_run_request(&profile, &[], &[]).expect_err("unknown file type should fail");

        assert!(error.contains("Profile 'bad-type'"));
        assert!(error.contains("Unknown file type group 'Unknown'"));
        Ok(())
    }
}
