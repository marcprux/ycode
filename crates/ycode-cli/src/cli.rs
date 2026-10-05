// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use crate::res;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ycode", version, about = res::str::about())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand)]
pub enum Command {
    #[command(about = res::str::project_about())]
    Project(ProjectArgs),
}
#[derive(Args)]
pub struct ProjectArgs {
    #[arg(short = 'p', long = "project", global = true, default_value = "project.xcproj", help = res::str::path_help())]
    pub path: PathBuf,
    #[arg(long, global = true, help = res::str::json_help())]
    pub json: bool,
    #[command(subcommand)]
    pub command: ProjectCommand,
}
#[derive(Subcommand)]
pub enum ProjectCommand {
    #[command(about = res::str::info_help())]
    Info,
    #[command(about = res::str::targets_help())]
    Targets,
    #[command(about = res::str::files_help())]
    Files,
    #[command(about = res::str::packages_help())]
    Packages,
    #[command(about = res::str::configurations_help())]
    Configurations,
    #[command(about = res::str::validate_help())]
    Validate,
    #[command(about = res::str::get_help())]
    Get {
        #[arg(help = res::str::pointer_help())]
        pointer: String,
    },
    #[command(about = res::str::set_help())]
    Set {
        #[arg(help = res::str::pointer_help())]
        pointer: String,
        #[arg(allow_hyphen_values = true, help = res::str::value_help())]
        value: String,
        #[arg(long, help = res::str::string_help())]
        string: bool,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = res::str::remove_help())]
    Remove {
        #[arg(help = res::str::pointer_help())]
        pointer: String,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = res::str::settings_help())]
    Settings {
        #[arg(long, global = true, help = res::str::target_help())]
        target: Option<String>,
        #[command(subcommand)]
        command: SettingsCommand,
    },
    #[command(about = res::str::export_help())]
    Export,
}
#[derive(Subcommand)]
pub enum SettingsCommand {
    #[command(about = res::str::settings_list_help())]
    List,
    #[command(about = res::str::settings_set_help())]
    Set {
        #[arg(help = res::str::key_help())]
        key: String,
        #[arg(allow_hyphen_values = true, help = res::str::setting_value_help())]
        value: String,
        #[command(flatten)]
        output: Output,
    },
    #[command(about = res::str::settings_remove_help())]
    Remove {
        #[arg(help = res::str::key_help())]
        key: String,
        #[command(flatten)]
        output: Output,
    },
}
#[derive(Args)]
pub struct Output {
    #[arg(long, conflicts_with = "output", help = res::str::write_help())]
    pub write: bool,
    #[arg(short, long, help = res::str::output_help())]
    pub output: Option<PathBuf>,
}
