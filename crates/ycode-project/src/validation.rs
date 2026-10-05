// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use crate::{Error, Project, Result, model::*, res};
use std::collections::BTreeSet;

impl Project {
    /// Check supported capabilities and invariants enforced by this model layer.
    /// This does not resolve files, build settings, or remote dependency graphs.
    pub fn validate(&self) -> Result<()> {
        for capability in &self.required_capabilities {
            if capability.as_ref() != "known capability for testing" {
                return Err(Error::UnsupportedCapability(capability.to_string()));
            }
        }
        let mut names = BTreeSet::new();
        for target in &self.targets {
            if !names.insert(target.name()) {
                return Err(Error::DuplicateTarget(target.name().into()));
            }
            for config in &target.common().specialized_configurations {
                if matches!(config, Configuration::Name(_))
                    || matches!(config, Configuration::Detailed(c) if c.id.is_none() && c.file.is_none())
                {
                    return Err(Error::Schema(res::str::redundant_configuration().into()));
                }
            }
        }
        for package in &self.packages {
            if let SwiftPackageLocation::Remote {
                version: Some(SwiftPackageVersionConstraint::VersionRange { range }),
                ..
            } = &package.location
                && range.split("..<").count() != 2
            {
                return Err(Error::Schema(res::str::invalid_range().into()));
            }
        }
        fn references(refs: &[Reference]) -> Result<()> {
            for reference in refs {
                match reference {
                    Reference::Group(group) => references(&group.children)?,
                    Reference::Folder(folder) => {
                        for set in &folder.membership_exceptions {
                            let common = match set {
                                FolderExceptionSet::Target(s) => &s.common,
                                FolderExceptionSet::BuildPhase(s) => &s.common,
                            };
                            if common.inclusions.is_some() && common.exclusions.is_some() {
                                return Err(Error::Schema(
                                    res::str::conflicting_exceptions().into(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        references(&self.files)
    }

    /// Parse and validate a semantic model. Use ProjectDocument to keep source text.
    pub fn from_json5(source: &str) -> Result<Self> {
        let (_, value) = crate::syntax::parse(source)?;
        let project: Self = serde_json::from_value(value)?;
        project.validate()?;
        Ok(project)
    }

    /// Generate fresh JSON (also valid JSON5). Original formatting is not retained.
    pub fn to_json_pretty(&self) -> Result<String> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029")
            + "\n")
    }
}
