// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ycode",
    version,
    about = "Inspect and edit Xcode JSON5 projects"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Read and update project.xcproj files")]
    Project(ProjectArgs),
}
#[derive(Args)]
pub struct ProjectArgs {
    #[arg(
        short = 'p',
        long = "project",
        global = true,
        default_value = "project.xcproj",
        help = "Project file or .xcodeproj bundle"
    )]
    pub path: PathBuf,
    #[arg(long, global = true, help = "Output machine-readable JSON")]
    pub json: bool,
    #[command(subcommand)]
    pub command: ProjectCommand,
}
#[derive(Subcommand)]
pub enum ProjectCommand {
    #[command(about = "Summarize the project")]
    Info,
    #[command(about = "List targets and product types")]
    Targets,
    #[command(about = "List entries in the project file tree")]
    Files,
    #[command(about = "List Swift package dependencies")]
    Packages,
    #[command(about = "List build configurations")]
    Configurations,
    #[command(about = "Validate the supported project schema and invariants")]
    Validate,
    #[command(about = "Extract a value using an RFC 6901 JSON pointer")]
    Get {
        #[arg(help = "RFC 6901 pointer (escape ~ as ~0 and / as ~1)")]
        pointer: String,
    },
    #[command(about = "Set a JSON5 value at a pointer; preview on stdout by default")]
    Set {
        #[arg(help = "RFC 6901 pointer (escape ~ as ~0 and / as ~1)")]
        pointer: String,
        #[arg(
            allow_hyphen_values = true,
            help = "JSON5 value; quote strings or use --string"
        )]
        value: String,
        #[arg(long, help = "Treat VALUE as a literal string")]
        string: bool,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = "Remove a value at a pointer; preview on stdout by default")]
    Remove {
        #[arg(help = "RFC 6901 pointer (escape ~ as ~0 and / as ~1)")]
        pointer: String,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = "Inspect or update project or target build settings")]
    Settings {
        #[arg(
            long,
            global = true,
            help = "Target name (omit for project-level settings)"
        )]
        target: Option<String>,
        #[command(subcommand)]
        command: SettingsCommand,
    },
    #[command(about = "Export the semantic model as formatted JSON (discards source formatting)")]
    Export,
}
#[derive(Subcommand)]
pub enum SettingsCommand {
    #[command(about = "List build settings without evaluating Xcode conditionals")]
    List,
    #[command(about = "Set a string build setting")]
    Set {
        #[arg(help = "Build-setting key, including any Xcode condition suffix")]
        key: String,
        #[arg(allow_hyphen_values = true, help = "Literal build-setting string")]
        value: String,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = "Remove a build setting")]
    Remove {
        #[arg(help = "Build-setting key, including any Xcode condition suffix")]
        key: String,
        #[command(flatten)]
        output: Output,
    },
}
#[derive(Args)]
pub struct Output {
    #[arg(
        long,
        conflicts_with = "output",
        help = "Atomically update the input after checking for external changes"
    )]
    pub write: bool,
    #[arg(
        short,
        long,
        help = "Write to a new file; refuses an existing destination"
    )]
    pub output: Option<PathBuf>,
}
