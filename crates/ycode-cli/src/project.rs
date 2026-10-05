// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use crate::{
    cli::{Output, ProjectArgs, ProjectCommand, SettingsCommand},
    res,
};
use serde_json::{Value, json};
use std::io::{self, Write};
use ycode_project::{Error, ProjectDocument, Reference, Result};

pub fn run(args: ProjectArgs) -> Result<()> {
    let mut document = ProjectDocument::open(&args.path)?;
    match args.command {
        ProjectCommand::Info => {
            let p = document.project();
            let count = file_entries(&p.files).len();
            if args.json {
                print_json(&json!({"targets":p.targets.len(),"file-tree-entries":count,
                    "packages":p.packages.len(),"configurations":p.configurations.len(),
                    "default-configuration":p.default_configuration,"development-language":p.localizations.development}))
            } else {
                print_line(
                    &res::str::summary()
                        .replace("{targets}", &p.targets.len().to_string())
                        .replace("{files}", &count.to_string())
                        .replace("{packages}", &p.packages.len().to_string())
                        .replace("{configurations}", &p.configurations.len().to_string())
                        .replace("{default}", p.default_configuration.as_ref())
                        .replace("{language}", p.localizations.development.as_ref()),
                )
            }
        }
        ProjectCommand::Targets => {
            if args.json {
                print_json(&serde_json::to_value(&document.project().targets)?)
            } else {
                for t in &document.project().targets {
                    print_line(&format!(
                        "{}\t{}",
                        t.name(),
                        t.common()
                            .product_type_id()
                            .map(|p| p.0)
                            .unwrap_or_default()
                    ))?;
                }
                Ok(())
            }
        }
        ProjectCommand::Files => print_json(&Value::Array(file_entries(&document.project().files))),
        ProjectCommand::Packages => {
            print_json(&serde_json::to_value(&document.project().packages)?)
        }
        ProjectCommand::Configurations => {
            print_json(&serde_json::to_value(&document.project().configurations)?)
        }
        ProjectCommand::Validate => {
            if args.json {
                print_json(&json!({"valid":true}))
            } else {
                print_line(res::str::valid())
            }
        }
        ProjectCommand::Get { pointer } => print_json(document.get(&pointer)?),
        ProjectCommand::Set {
            pointer,
            value,
            string,
            output,
        } => {
            let value = if string {
                Value::String(value)
            } else {
                ycode_project::parse_json5_value(&value)?
            };
            document.set(&pointer, value)?;
            finish(&mut document, output)
        }
        ProjectCommand::Remove { pointer, output } => {
            document.remove(&pointer)?;
            finish(&mut document, output)
        }
        ProjectCommand::Settings { target, command } => match command {
            SettingsCommand::List => {
                let p = document.project();
                let settings = if let Some(name) = &target {
                    &p.targets
                        .iter()
                        .find(|t| t.name() == name)
                        .ok_or_else(|| Error::TargetNotFound(name.clone()))?
                        .common()
                        .build_settings
                } else {
                    &p.build_settings
                };
                print_json(&serde_json::to_value(settings)?)
            }
            SettingsCommand::Set { key, value, output } => {
                document.set_build_setting(target.as_deref(), &key, value.into())?;
                finish(&mut document, output)
            }
            SettingsCommand::Remove { key, output } => {
                document.remove_build_setting(target.as_deref(), &key)?;
                finish(&mut document, output)
            }
        },
        ProjectCommand::Export => print_text(&document.project().to_json_pretty()?),
    }
}
fn finish(document: &mut ProjectDocument, output: Output) -> Result<()> {
    if output.write {
        document.save()
    } else if let Some(path) = output.output {
        document.write_new(path)
    } else {
        print_text(document.source())
    }
}
fn print_text(text: &str) -> Result<()> {
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(Into::into)
}
fn print_line(text: &str) -> Result<()> {
    print_text(&format!("{text}\n"))
}
fn print_json(value: &Value) -> Result<()> {
    print_line(&serde_json::to_string_pretty(value)?)
}
fn file_entries(references: &[Reference]) -> Vec<Value> {
    fn walk(reference: &Reference, parents: &[String], output: &mut Vec<Value>) {
        let mut components = parents.to_vec();
        components.push(reference.name().into());
        output
            .push(json!({"name-path":components,"path":reference.path().to_string(),"kind":reference.kind()}));
        match reference {
            Reference::Group(group) => {
                for child in &group.children {
                    walk(child, &components, output);
                }
            }
            Reference::VariantGroup(group) => {
                for child in &group.children {
                    walk(
                        &Reference::FileReference(child.clone()),
                        &components,
                        output,
                    );
                }
            }
            Reference::VersionGroup(group) => {
                for child in &group.children {
                    walk(
                        &Reference::FileReference(child.clone()),
                        &components,
                        output,
                    );
                }
            }
            _ => {}
        }
    }
    let mut entries = Vec::new();
    for reference in references {
        walk(reference, &[], &mut entries);
    }
    entries
}
