// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SwiftPackageProductType {
    #[default]
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "build-tool-plugin")]
    BuildToolPlugin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SwiftPackageLocation {
    Local {
        path: String,
    },
    Remote {
        repository: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        version: Option<SwiftPackageVersionConstraint>,
    },
}
/// SwiftPackage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct SwiftPackage {
    #[serde(flatten)]
    pub location: SwiftPackageLocation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// The upstream version selector object (including both range encodings).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SwiftPackageVersionConstraint {
    Revision {
        revision: String,
    },
    Branch {
        branch: String,
    },
    Version {
        version: String,
    },
    UpToNextMinorVersion {
        #[serde(rename = "up-to-next-minor-version")]
        version: String,
    },
    UpToNextMajorVersion {
        #[serde(rename = "up-to-next-major-version")]
        version: String,
    },
    VersionRange {
        #[serde(rename = "version-range")]
        range: String,
    },
    ExplicitVersionRange {
        #[serde(rename = "version-range-min")]
        min: String,
        #[serde(rename = "version-range-max")]
        max: String,
    },
}
/// SwiftPackageProductReference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SwiftPackageProductReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<SwiftPackageName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub product_name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub product_type: SwiftPackageProductType,
}

/// SwiftPackageProductTargetMember.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct SwiftPackageProductTargetMember {
    #[serde(flatten)]
    pub package_product: SwiftPackageProductReference,
    pub build_phase: TargetBuildFile,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// RemoteTarget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteTarget {
    pub project: GroupTreeReference,
    pub target: String,
    pub target_id: ObjectId,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// RemoteProduct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteProduct {
    #[serde(alias = "name")]
    pub path: String,
    pub project: GroupTreeReference,
    pub target: String,
    pub product_id: ObjectId,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileTypeId>,
    pub target_membership: Vec<ProjectBuildFile>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

impl TryFrom<Value> for SwiftPackage {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            location: SwiftPackageLocation,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            traits: Vec<String>,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "kind" | "path" | "repository" | "traits" | "version"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            location: wire.location,
            traits: wire.traits,
            extra,
        })
    }
}

impl TryFrom<Value> for SwiftPackageProductTargetMember {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            package_product: SwiftPackageProductReference,
            build_phase: TargetBuildFile,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "build-phase" | "id" | "package" | "product-name" | "product-type"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            package_product: wire.package_product,
            build_phase: wire.build_phase,
            extra,
        })
    }
}
