// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineEnding {
    #[serde(rename = "line-feed")]
    LineFeed,
    #[serde(rename = "carriage-return")]
    CarriageReturn,
    #[serde(rename = "carriage-return-line-feed")]
    CarriageReturnLineFeed,
    #[serde(rename = "preserve")]
    Preserve,
}

/// FileReference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct FileReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub path: FilePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<bool>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileTypeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<TextEncoding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_ending: Option<LineEnding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_membership: Vec<ProjectBuildFile>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// A virtual group; an omitted name is derived from its path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub path: FilePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Reference>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// Folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Folder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub path: FilePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<bool>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub target_membership: BTreeSet<LocalTargetReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub membership_exceptions: Vec<FolderExceptionSet>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub file_types: BTreeMap<FolderMemberId, FileTypeId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub opaque_folders: BTreeSet<FolderMemberId>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// VariantGroup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct VariantGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub path: FilePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_membership: Vec<ProjectBuildFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FileReference>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// VersionGroup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct VersionGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub path: FilePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_version: Option<GroupTreeReference>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileTypeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_membership: Vec<ProjectBuildFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FileReference>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

default_tagged_enum! {
    /// An item in the project's files tree. The omitted kind means file-reference.
    pub enum Reference, "file-reference" {
        FileReference(FileReference) => "file-reference",
        Group(Group) => "group",
        Folder(Folder) => "folder",
        VariantGroup(VariantGroup) => "variant-group",
        VersionGroup(VersionGroup) => "version-group"
    }
}
impl Reference {
    pub fn path(&self) -> &FilePath {
        match self {
            Self::FileReference(v) => &v.path,
            Self::Group(v) => &v.path,
            Self::Folder(v) => &v.path,
            Self::VariantGroup(v) => &v.path,
            Self::VersionGroup(v) => &v.path,
        }
    }
    pub fn name(&self) -> &str {
        let explicit = match self {
            Self::Group(v) => v.name.as_deref(),
            Self::VariantGroup(v) => v.name.as_deref(),
            Self::VersionGroup(v) => v.name.as_deref(),
            _ => None,
        };
        explicit.unwrap_or_else(|| {
            self.path()
                .path()
                .rsplit('/')
                .find(|s| !s.is_empty())
                .unwrap_or("")
        })
    }
}
/// CommonExceptionSetProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CommonExceptionSetProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inclusions: Option<BTreeSet<FolderMemberId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusions: Option<BTreeSet<FolderMemberId>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub platforms: BTreeMap<FolderMemberId, BTreeSet<PlatformFilter>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<FolderMemberId, BuildFileAttributes>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub asset_tags: BTreeMap<FolderMemberId, BTreeSet<AssetTag>>,
}

/// TargetExceptionSet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct TargetExceptionSet {
    pub target: LocalTargetReference,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub public_headers: BTreeSet<FolderMemberId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub private_headers: BTreeSet<FolderMemberId>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub compiler_flags: BTreeMap<FolderMemberId, String>,
    #[serde(flatten)]
    pub common: CommonExceptionSetProperties,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// BuildPhaseExceptionSet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct BuildPhaseExceptionSet {
    pub build_phase: ProjectBuildPhaseReference,
    #[serde(flatten)]
    pub common: CommonExceptionSetProperties,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FolderExceptionSet {
    Target(TargetExceptionSet),
    BuildPhase(BuildPhaseExceptionSet),
}

impl TryFrom<Value> for TargetExceptionSet {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            target: LocalTargetReference,
            #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
            public_headers: BTreeSet<FolderMemberId>,
            #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
            private_headers: BTreeSet<FolderMemberId>,
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            compiler_flags: BTreeMap<FolderMemberId, String>,
            #[serde(flatten)]
            common: CommonExceptionSetProperties,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "asset-tags"
                        | "attributes"
                        | "compiler-flags"
                        | "exclusions"
                        | "inclusions"
                        | "platforms"
                        | "private-headers"
                        | "public-headers"
                        | "target"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            target: wire.target,
            public_headers: wire.public_headers,
            private_headers: wire.private_headers,
            compiler_flags: wire.compiler_flags,
            common: wire.common,
            extra,
        })
    }
}

impl TryFrom<Value> for BuildPhaseExceptionSet {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            build_phase: ProjectBuildPhaseReference,
            #[serde(flatten)]
            common: CommonExceptionSetProperties,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "asset-tags"
                        | "attributes"
                        | "build-phase"
                        | "exclusions"
                        | "inclusions"
                        | "platforms"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            build_phase: wire.build_phase,
            common: wire.common,
            extra,
        })
    }
}

/// The five supported entries in a project's files tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceKind {
    #[default]
    FileReference,
    Group,
    Folder,
    VariantGroup,
    VersionGroup,
}
impl Reference {
    pub fn kind(&self) -> ReferenceKind {
        match self {
            Self::FileReference(_) => ReferenceKind::FileReference,
            Self::Group(_) => ReferenceKind::Group,
            Self::Folder(_) => ReferenceKind::Folder,
            Self::VariantGroup(_) => ReferenceKind::VariantGroup,
            Self::VersionGroup(_) => ReferenceKind::VersionGroup,
        }
    }
}
