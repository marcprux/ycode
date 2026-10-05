// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeaderRole {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachInterfaceGeneration {
    #[serde(rename = "client")]
    Client,
    #[serde(rename = "server")]
    Server,
    #[serde(rename = "both")]
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeGenerationVisibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "project")]
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CodeGeneration {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "skip")]
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HeaderPreservation {
    #[default]
    #[serde(rename = "keep")]
    Keep,
    #[serde(rename = "remove-on-copy")]
    RemoveOnCopy,
}

/// BuildFileAttributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFileAttributes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_role: Option<HeaderRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mach_interface_generation: Option<MachInterfaceGeneration>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub is_weak: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub code_sign_on_copy: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub code_generation: CodeGeneration,
    #[serde(default, skip_serializing_if = "is_default")]
    pub header_preservation: HeaderPreservation,
    #[serde(default, skip_serializing_if = "is_default")]
    pub decompress: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_generation_visibility: Option<CodeGenerationVisibility>,
}

/// BuildFileProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFileProperties {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub platforms: BTreeSet<PlatformFilter>,
    #[serde(flatten)]
    pub attributes: BuildFileAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub asset_tags: BTreeSet<AssetTag>,
}

/// ProjectBuildFileDetails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct ProjectBuildFileDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub build_phase: ProjectBuildPhaseReference,
    #[serde(flatten)]
    pub properties: BuildFileProperties,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// Compact or expanded project-wide build membership.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ProjectBuildFile {
    Compact(ProjectBuildPhaseReference),
    Detailed(ProjectBuildFileDetails),
}
/// TargetBuildFile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct TargetBuildFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub build_phase: TargetBuildPhaseReference,
    #[serde(flatten)]
    pub properties: BuildFileProperties,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

impl TryFrom<Value> for ProjectBuildFileDetails {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            id: Option<ObjectId>,
            build_phase: ProjectBuildPhaseReference,
            #[serde(flatten)]
            properties: BuildFileProperties,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "arguments"
                        | "asset-tags"
                        | "build-phase"
                        | "code-generation"
                        | "code-generation-visibility"
                        | "code-sign-on-copy"
                        | "decompress"
                        | "header-preservation"
                        | "header-role"
                        | "id"
                        | "is-weak"
                        | "mach-interface-generation"
                        | "platforms"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            id: wire.id,
            build_phase: wire.build_phase,
            properties: wire.properties,
            extra,
        })
    }
}

impl TryFrom<Value> for TargetBuildFile {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            id: Option<ObjectId>,
            build_phase: TargetBuildPhaseReference,
            #[serde(flatten)]
            properties: BuildFileProperties,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "arguments"
                        | "asset-tags"
                        | "build-phase"
                        | "code-generation"
                        | "code-generation-visibility"
                        | "code-sign-on-copy"
                        | "decompress"
                        | "header-preservation"
                        | "header-role"
                        | "id"
                        | "is-weak"
                        | "mach-interface-generation"
                        | "platforms"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            id: wire.id,
            build_phase: wire.build_phase,
            properties: wire.properties,
            extra,
        })
    }
}

impl Serialize for ProjectBuildFile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Detailed(details) => details.serialize(serializer),
            Self::Compact(reference) => {
                let value = serde_json::to_value(reference).map_err(serde::ser::Error::custom)?;
                if value.is_string() {
                    value.serialize(serializer)
                } else {
                    serde_json::json!({"build-phase": value}).serialize(serializer)
                }
            }
        }
    }
}
