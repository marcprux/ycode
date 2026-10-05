// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildPhaseKind {
    #[serde(rename = "apple-script")]
    AppleScript,
    #[serde(rename = "frameworks")]
    Frameworks,
    #[serde(rename = "headers")]
    Headers,
    #[serde(rename = "java-archive")]
    JavaArchive,
    #[serde(rename = "resources")]
    Resources,
    #[serde(rename = "rez")]
    Rez,
    #[serde(rename = "compile-sources")]
    Sources,
    #[serde(rename = "copy")]
    Copy,
    #[serde(rename = "script")]
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BuildPhaseScope {
    #[default]
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "install")]
    Install,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BundleBasePath {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "build-products-directory")]
    BuildProductsDirectory,
    #[serde(rename = "shared-frameworks-directory")]
    SharedFrameworksDirectory,
    #[serde(rename = "shared-support-directory")]
    SharedSupportDirectory,
    #[serde(rename = "java-directory")]
    JavaDirectory,
    #[serde(rename = "frameworks-directory")]
    FrameworksDirectory,
    #[serde(rename = "resources-directory")]
    ResourcesDirectory,
    #[serde(rename = "package-info-file")]
    PackageInfoFile,
    #[serde(rename = "apple-scripts-directory")]
    AppleScriptsDirectory,
    #[serde(rename = "plugins-directory")]
    PluginsDirectory,
    #[serde(rename = "private-headers-directory")]
    PrivateHeadersDirectory,
    #[serde(rename = "headers-directory")]
    HeadersDirectory,
    #[serde(rename = "contents-directory")]
    ContentsDirectory,
    #[serde(rename = "executables-directory")]
    ExecutablesDirectory,
    #[serde(rename = "info-plist-file")]
    InfoPlistFile,
    #[serde(rename = "main-executable-file")]
    MainExecutableFile,
    #[serde(rename = "shallow-main-executable-file")]
    ShallowMainExecutableFile,
}

/// Properties shared by all build phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BuildPhaseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// AppleScriptBuildPhaseProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct AppleScriptBuildPhaseProperties {
    #[serde(flatten)]
    pub base: BuildPhaseProperties,
    #[serde(default, skip_serializing_if = "is_default")]
    pub is_shared_context: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub context_name: String,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// CopyFilesBuildPhaseProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct CopyFilesBuildPhaseProperties {
    #[serde(flatten)]
    pub base: BuildPhaseProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_base_path: Option<BundleBasePath>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub relative_path: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub scope: BuildPhaseScope,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// ScriptBuildPhaseProperties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "Value")]
pub struct ScriptBuildPhaseProperties {
    #[serde(flatten)]
    pub base: BuildPhaseProperties,
    pub shell: String,
    pub script: MultilineText,
    #[serde(default, skip_serializing_if = "is_default")]
    pub log_environment_variables: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_file_list_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_file_list_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_file: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub run_on_every_build: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub scope: BuildPhaseScope,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

/// Build phase, with compact string encodings accepted for the six simple phases.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum BuildPhase {
    #[serde(rename = "apple-script")]
    AppleScript(AppleScriptBuildPhaseProperties),
    #[serde(rename = "frameworks")]
    Frameworks(BuildPhaseProperties),
    #[serde(rename = "headers")]
    Headers(BuildPhaseProperties),
    #[serde(rename = "java-archive")]
    JavaArchive(BuildPhaseProperties),
    #[serde(rename = "resources")]
    Resources(BuildPhaseProperties),
    #[serde(rename = "rez")]
    Rez(BuildPhaseProperties),
    #[serde(rename = "compile-sources")]
    Sources(BuildPhaseProperties),
    #[serde(rename = "copy")]
    Copy(CopyFilesBuildPhaseProperties),
    #[serde(rename = "script")]
    Script(ScriptBuildPhaseProperties),
}
impl<'de> Deserialize<'de> for BuildPhase {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut value = Value::deserialize(deserializer)?;
        if let Value::String(kind) = &value {
            if matches!(kind.as_str(), "script" | "copy" | "apple-script") {
                return Err(serde::de::Error::custom(crate::res::str::phase_properties()));
            }
            value = serde_json::json!({"kind": kind});
        }
        #[derive(Deserialize)]
        #[serde(tag = "kind")]
        enum Wire {
            #[serde(rename = "apple-script")]
            AppleScript(AppleScriptBuildPhaseProperties),
            #[serde(rename = "frameworks")]
            Frameworks(BuildPhaseProperties),
            #[serde(rename = "headers")]
            Headers(BuildPhaseProperties),
            #[serde(rename = "java-archive")]
            JavaArchive(BuildPhaseProperties),
            #[serde(rename = "resources")]
            Resources(BuildPhaseProperties),
            #[serde(rename = "rez")]
            Rez(BuildPhaseProperties),
            #[serde(rename = "compile-sources")]
            Sources(BuildPhaseProperties),
            #[serde(rename = "copy")]
            Copy(CopyFilesBuildPhaseProperties),
            #[serde(rename = "script")]
            Script(ScriptBuildPhaseProperties),
        }
        let wire: Wire = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(match wire {
            Wire::AppleScript(p) => Self::AppleScript(p),
            Wire::Frameworks(p) => Self::Frameworks(p),
            Wire::Headers(p) => Self::Headers(p),
            Wire::JavaArchive(p) => Self::JavaArchive(p),
            Wire::Resources(p) => Self::Resources(p),
            Wire::Rez(p) => Self::Rez(p),
            Wire::Sources(p) => Self::Sources(p),
            Wire::Copy(p) => Self::Copy(p),
            Wire::Script(p) => Self::Script(p),
        })
    }
}
impl BuildPhase {
    pub fn kind(&self) -> BuildPhaseKind {
        match self {
            Self::AppleScript(_) => BuildPhaseKind::AppleScript,
            Self::Frameworks(_) => BuildPhaseKind::Frameworks,
            Self::Headers(_) => BuildPhaseKind::Headers,
            Self::JavaArchive(_) => BuildPhaseKind::JavaArchive,
            Self::Resources(_) => BuildPhaseKind::Resources,
            Self::Rez(_) => BuildPhaseKind::Rez,
            Self::Sources(_) => BuildPhaseKind::Sources,
            Self::Copy(_) => BuildPhaseKind::Copy,
            Self::Script(_) => BuildPhaseKind::Script,
        }
    }
}
/// BuildRule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub processor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileTypeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_patterns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<MultilineText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_file_lists: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_file_lists: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_files_compiler_flags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_file: Option<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub run_once_per_architecture: bool,
    /// Fields introduced by future format revisions.
    #[serde(flatten)]
    pub extra: ExtraFields,
}

impl TryFrom<Value> for AppleScriptBuildPhaseProperties {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            base: BuildPhaseProperties,
            #[serde(default, skip_serializing_if = "is_default")]
            is_shared_context: bool,
            #[serde(default, skip_serializing_if = "is_default")]
            context_name: String,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "context-name" | "id" | "is-shared-context" | "name"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            base: wire.base,
            is_shared_context: wire.is_shared_context,
            context_name: wire.context_name,
            extra,
        })
    }
}

impl TryFrom<Value> for CopyFilesBuildPhaseProperties {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            base: BuildPhaseProperties,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            bundle_base_path: Option<BundleBasePath>,
            #[serde(default, skip_serializing_if = "is_default")]
            relative_path: String,
            #[serde(default, skip_serializing_if = "is_default")]
            scope: BuildPhaseScope,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "bundle-base-path" | "id" | "name" | "relative-path" | "scope"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            base: wire.base,
            bundle_base_path: wire.bundle_base_path,
            relative_path: wire.relative_path,
            scope: wire.scope,
            extra,
        })
    }
}

impl TryFrom<Value> for ScriptBuildPhaseProperties {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Wire {
            #[serde(flatten)]
            base: BuildPhaseProperties,
            shell: String,
            script: MultilineText,
            #[serde(default, skip_serializing_if = "is_default")]
            log_environment_variables: bool,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            input_paths: Vec<String>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            input_file_list_paths: Vec<String>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            output_paths: Vec<String>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            output_file_list_paths: Vec<String>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            dependency_file: Option<String>,
            #[serde(default, skip_serializing_if = "is_default")]
            run_on_every_build: bool,
            #[serde(default, skip_serializing_if = "is_default")]
            scope: BuildPhaseScope,
        }
        let wire: Wire = serde_json::from_value(value.clone())?;
        let extra = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "dependency-file"
                        | "id"
                        | "input-file-list-paths"
                        | "input-paths"
                        | "log-environment-variables"
                        | "name"
                        | "output-file-list-paths"
                        | "output-paths"
                        | "run-on-every-build"
                        | "scope"
                        | "script"
                        | "shell"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Ok(Self {
            base: wire.base,
            shell: wire.shell,
            script: wire.script,
            log_environment_variables: wire.log_environment_variables,
            input_paths: wire.input_paths,
            input_file_list_paths: wire.input_file_list_paths,
            output_paths: wire.output_paths,
            output_file_list_paths: wire.output_file_list_paths,
            dependency_file: wire.dependency_file,
            run_on_every_build: wire.run_on_every_build,
            scope: wire.scope,
            extra,
        })
    }
}
