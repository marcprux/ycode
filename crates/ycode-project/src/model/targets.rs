// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegacyProvisioningStyle {
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "manual")]
    Manual,
}

/// CommonTargetProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CommonTargetProperties {
    pub name: String,
    pub id: ObjectId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_list_debug_id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product: Option<GroupTreeReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_product_type: Option<ProductTypeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_host_target: Option<LocalTargetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_provisioning_style: Option<LegacyProvisioningStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_swift_update: Option<MarketingVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_swift_migration: Option<MarketingVersion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TargetDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_phases: Vec<BuildPhase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_rules: Vec<BuildRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub specialized_configurations: Vec<Configuration>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub build_settings: BTreeMap<String, BuildSetting>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package_product_members: Vec<SwiftPackageProductTargetMember>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// ExternalBuildSystemTargetProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExternalBuildSystemTargetProperties {
    pub build_tool_path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub build_tool_arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_tool_working_directory: Option<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub pass_build_settings_in_environment: bool,
    #[serde(flatten)]
    pub common: CommonTargetProperties,
}

default_tagged_enum! {
    /// A buildable target. Native is the default kind.
    pub enum Target, "native" {
        Native(CommonTargetProperties) => "native",
        Aggregate(CommonTargetProperties) => "aggregate",
        ExternalBuildSystem(ExternalBuildSystemTargetProperties) => "external-build-system"
    }
}
impl Target {
    pub fn common(&self) -> &CommonTargetProperties {
        match self {
            Self::Native(p) | Self::Aggregate(p) => p,
            Self::ExternalBuildSystem(p) => &p.common,
        }
    }
    pub fn common_mut(&mut self) -> &mut CommonTargetProperties {
        match self {
            Self::Native(p) | Self::Aggregate(p) => p,
            Self::ExternalBuildSystem(p) => &mut p.common,
        }
    }
    pub fn name(&self) -> &str {
        &self.common().name
    }
}
impl CommonTargetProperties {
    pub fn product_type_id(&self) -> Option<ProductTypeId> {
        self.product_type
            .as_ref()
            .map(|s| ProductTypeId(format!("com.apple.product-type.{s}")))
            .or_else(|| self.full_product_type.clone())
    }
}
/// Compact local dependency or a dependency with filters / remote location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetDependency {
    Local(LocalTargetReference),
    Detailed(TargetDependencyDetails),
}
default_tagged_enum! {
    pub enum TargetDependencyDetails, "localTarget" {
        Local(LocalTargetDependency) => "localTarget",
        Remote(RemoteTargetDependency) => "remoteTarget",
        Package(PackageTargetDependency) => "package"
    }
}
/// LocalTargetDependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LocalTargetDependency {
    pub target: LocalTargetReference,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub platforms: BTreeSet<PlatformFilter>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// RemoteTargetDependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteTargetDependency {
    pub project: GroupTreeReference,
    pub target: String,
    pub target_id: ObjectId,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub platforms: BTreeSet<PlatformFilter>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// PackageTargetDependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct PackageTargetDependency {
    #[serde(flatten)]
    pub product: SwiftPackageProductReference,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub platforms: BTreeSet<PlatformFilter>,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

impl TryFrom<Value> for PackageTargetDependency {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            product: SwiftPackageProductReference,
            #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
            platforms: BTreeSet<PlatformFilter>,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "id" | "package" | "platforms" | "product-name" | "product-type"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            product: wire.product,
            platforms: wire.platforms,
            extra,
        })
    }
}

/// The three supported target kinds; absent kind fields mean Native.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    #[default]
    Native,
    Aggregate,
    ExternalBuildSystem,
}
impl Target {
    pub fn kind(&self) -> TargetKind {
        match self {
            Self::Native(_) => TargetKind::Native,
            Self::Aggregate(_) => TargetKind::Aggregate,
            Self::ExternalBuildSystem(_) => TargetKind::ExternalBuildSystem,
        }
    }
}
