//! # Main Entry Point
//!
//! This file is the main executable for the `code-file-wrapper` program. It
//! reads command line arguments, optionally opens a folder picker dialog, and
//! then generates an output file containing tag-wrapped file contents.

mod cli;
mod file_ops;

use crate::cli::CliArgs;
use crate::file_ops::write_folder_tags;
use clap::Parser;
use rfd::FileDialog;
use std::path::PathBuf;

/// Main entry point for `code-file-wrapper`.
///
/// If a valid path is provided, that folder is immediately processed.
/// Otherwise, a folder picker dialog is opened to allow folder selection.
fn main() {
    let args = CliArgs::parse();
    let folder = if let Some(ref path) = args.path {
        let p = PathBuf::from(path);
        if !p.exists() {
            eprintln!("Provided path does not exist. Exiting.");
            std::process::exit(1);
        }
        p
    } else {
        let selected_dir = FileDialog::new().set_directory(".").pick_folder();

        let Some(dir) = selected_dir else {
            eprintln!("No directory selected. Exiting.");
            std::process::exit(0);
        };

        if !dir.is_dir() {
            eprintln!("Selected path is not a directory. Exiting.");
            std::process::exit(1);
        }
        dir
    };

    if let Err(e) = write_folder_tags(&folder) {
        eprintln!("Error creating tags: {e}");
        std::process::exit(1);
    }
}
