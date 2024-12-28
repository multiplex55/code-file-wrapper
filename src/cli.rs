//! # CLI Module
//!
//! This file defines command line arguments for the `code-file-wrapper` program.
//! It uses the `clap` crate to parse optional path input for the application.

use clap::{ArgAction, Parser};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(
    name = "code-file-wrapper",
    version = "0.1.0",
    about = "Creates a text file of tag-wrapped file contents from a directory"
)]
pub struct CliArgs {
    /// Full path to a folder
    #[arg(action = ArgAction::Set, required = false)]
    pub path: Option<String>,
}
