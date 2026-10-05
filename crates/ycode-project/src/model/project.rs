// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// ProjectLocalizationInfo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectLocalizationInfo {
    pub development: Language,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub supported: BTreeSet<Language>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// A named configuration, optionally with an xcconfig anchor and stable ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Configuration {
    Name(ConfigurationName),
    Detailed(ConfigurationDetails),
}
impl Configuration {
    pub fn name(&self) -> &ConfigurationName {
        match self {
            Self::Name(n) => n,
            Self::Detailed(d) => &d.name,
        }
    }
}
/// ConfigurationDetails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationDetails {
    pub name: ConfigurationName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<GroupTreeAnchoredReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// The root project.xcproj data model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub files: Vec<Reference>,
    pub default_configuration: ConfigurationName,
    pub localizations: ProjectLocalizationInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_group_debug_id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_list_debug_id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_prefix: Option<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub build_independent_targets_in_parallel: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configurations: Vec<Configuration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<SwiftPackage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<Target>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub build_settings: BTreeMap<String, BuildSetting>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub required_capabilities: BTreeSet<Capability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imported_products: Vec<RemoteProduct>,
    #[serde(default = "default_products_group")]
    pub products_group: Option<GroupTreeReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_upgrade: Option<MarketingVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_swift_update: Option<MarketingVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_swift_migration: Option<MarketingVersion>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

fn default_products_group() -> Option<GroupTreeReference> {
    Some(GroupTreeReference::NamePath(NamePath::from("Products")))
}
impl Project {
    pub fn new(
        default_configuration: impl Into<ConfigurationName>,
        development: impl Into<Language>,
    ) -> Self {
        Self {
            files: vec![],
            default_configuration: default_configuration.into(),
            localizations: ProjectLocalizationInfo {
                development: development.into(),
                ..Default::default()
            },
            id: None,
            root_group_debug_id: None,
            configuration_list_debug_id: None,
            organization: None,
            class_prefix: None,
            build_independent_targets_in_parallel: true,
            configurations: vec![],
            packages: vec![],
            targets: vec![],
            build_settings: BTreeMap::new(),
            required_capabilities: BTreeSet::new(),
            imported_products: vec![],
            products_group: default_products_group(),
            last_upgrade: None,
            last_swift_update: None,
            last_swift_migration: None,
            extra: ExtraFields::new(),
        }
    }
}
