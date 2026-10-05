// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilePathBase {
    Absolute,
    Group,
    Project,
    Developer,
    BuildProducts,
    Sdk,
    SourceRoot(String),
}

/// A virtual base and path, independent of host operating system path conventions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePath {
    base: FilePathBase,
    path: String,
}
impl Default for FilePath {
    fn default() -> Self {
        Self {
            base: FilePathBase::Group,
            path: String::new(),
        }
    }
}
impl FilePath {
    pub fn new(base: FilePathBase, path: impl Into<String>) -> crate::Result<Self> {
        let path = path.into();
        if (path.starts_with('/') || path.starts_with('~')) != (base == FilePathBase::Absolute) {
            return Err(crate::Error::InvalidValue(path));
        }
        Ok(Self { base, path })
    }
    pub fn base(&self) -> &FilePathBase {
        &self.base
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}
fn unescape(s: &str, special: char) -> crate::Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(next) if next == special || next == '\\' => result.push(next),
                _ => return Err(crate::Error::InvalidValue(s.into())),
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}
fn escape(s: &str, special: char) -> String {
    s.chars()
        .flat_map(|c| {
            if c == special || c == '\\' {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
impl FromStr for FilePath {
    type Err = crate::Error;
    fn from_str(s: &str) -> crate::Result<Self> {
        if let Some(rest) = s.strip_prefix("<USER:") {
            let mut escaped = false;
            for (index, c) in rest.char_indices() {
                if c == '>' && !escaped {
                    let path = rest[index + 1..]
                        .strip_prefix('/')
                        .ok_or_else(|| crate::Error::InvalidValue(s.into()))?;
                    return Self::new(
                        FilePathBase::SourceRoot(unescape(&rest[..index], '>')?),
                        path,
                    );
                }
                escaped = c == '\\' && !escaped;
            }
            return Err(crate::Error::InvalidValue(s.into()));
        }
        if let Some(rest) = s.strip_prefix('<') {
            let (base, path) = rest
                .split_once(">/")
                .ok_or_else(|| crate::Error::InvalidValue(s.into()))?;
            let base = match base {
                "PROJECT" => FilePathBase::Project,
                "DEVELOPER" => FilePathBase::Developer,
                "PRODUCTS" => FilePathBase::BuildProducts,
                "SDK" => FilePathBase::Sdk,
                _ => return Err(crate::Error::InvalidValue(s.into())),
            };
            return Self::new(base, path);
        }
        Self::new(
            if s.starts_with('/') {
                FilePathBase::Absolute
            } else {
                FilePathBase::Group
            },
            unescape(s, '<')?,
        )
    }
}
impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = match &self.base {
            FilePathBase::Absolute | FilePathBase::Group => {
                return f.write_str(&escape(&self.path, '<'));
            }
            FilePathBase::Project => "PROJECT",
            FilePathBase::Developer => "DEVELOPER",
            FilePathBase::BuildProducts => "PRODUCTS",
            FilePathBase::Sdk => "SDK",
            FilePathBase::SourceRoot(name) => {
                return write!(f, "<USER:{}>/{}", escape(name, '>'), self.path);
            }
        };
        write!(f, "<{base}>/{}", self.path)
    }
}
impl Serialize for FilePath {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for FilePath {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamePathComponent {
    Current,
    Parent,
    Child(String),
}
impl NamePathComponent {
    fn compact(&self) -> Option<&str> {
        match self {
            Self::Current => Some("."),
            Self::Parent => Some(".."),
            Self::Child(s) if !s.contains('/') && s != "." && s != ".." => Some(s),
            _ => None,
        }
    }
    pub fn child_name(&self) -> Option<&str> {
        if let Self::Child(s) = self {
            Some(s)
        } else {
            None
        }
    }
}
impl Serialize for NamePathComponent {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if let Some(compact) = self.compact() {
            compact.serialize(s)
        } else if let Self::Child(name) = self {
            serde_json::json!({"name": name}).serialize(s)
        } else {
            unreachable!()
        }
    }
}
impl<'de> Deserialize<'de> for NamePathComponent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            String(String),
            Named { name: String },
        }
        match Wire::deserialize(d)? {
            Wire::Named { name } => Ok(Self::Child(name)),
            Wire::String(s) => match s.as_str() {
                "." => Ok(Self::Current),
                ".." => Ok(Self::Parent),
                _ if !s.contains('/') => Ok(Self::Child(s)),
                _ => Err(serde::de::Error::custom(crate::res::str::invalid_value())),
            },
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamePath {
    pub components: Vec<NamePathComponent>,
}
impl From<&str> for NamePath {
    fn from(s: &str) -> Self {
        Self {
            components: s
                .split('/')
                .map(|s| match s {
                    "." => NamePathComponent::Current,
                    ".." => NamePathComponent::Parent,
                    _ => NamePathComponent::Child(s.into()),
                })
                .collect(),
        }
    }
}
impl NamePath {
    pub fn compact(&self) -> Option<String> {
        if self.components.is_empty() {
            return None;
        }
        self.components
            .iter()
            .map(NamePathComponent::compact)
            .collect::<Option<Vec<_>>>()
            .map(|c| c.join("/"))
    }
}
impl Serialize for NamePath {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if let Some(compact) = self.compact() {
            compact.serialize(s)
        } else {
            self.components.serialize(s)
        }
    }
}
impl<'de> Deserialize<'de> for NamePath {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            String(String),
            Components(Vec<NamePathComponent>),
        }
        Ok(match Wire::deserialize(d)? {
            Wire::String(s) => Self::from(s.as_str()),
            Wire::Components(components) => Self { components },
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupTreeReference {
    ObjectId(ObjectId),
    NamePath(NamePath),
}
impl Serialize for GroupTreeReference {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ObjectId(id) => format!("id:{id}").serialize(s),
            Self::NamePath(path) => {
                if let Some(compact) = path.compact().filter(|s| !s.starts_with("id:")) {
                    compact.serialize(s)
                } else {
                    path.components.serialize(s)
                }
            }
        }
    }
}
impl<'de> Deserialize<'de> for GroupTreeReference {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        if let Some(id) = value.as_str().and_then(|s| s.strip_prefix("id:")) {
            return Ok(Self::ObjectId(id.into()));
        }
        serde_json::from_value(value)
            .map(Self::NamePath)
            .map_err(serde::de::Error::custom)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupTreeAnchoredReference {
    Anchor(GroupTreeReference),
    Relative {
        anchor: GroupTreeReference,
        #[serde(rename = "relative-path")]
        relative_path: NamePath,
    },
}

macro_rules! phase_reference {
    ($name:ident, $project:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
        #[serde(transparent)]
        pub struct $name(GroupTreeReference);
        impl $name {
            pub fn new(reference: GroupTreeReference) -> crate::Result<Self> {
                if let GroupTreeReference::NamePath(path) = &reference {
                    let offset = usize::from($project);
                    if !(1 + offset..=2 + offset).contains(&path.components.len())
                        || path.components.iter().any(|c| c.child_name().is_none())
                    {
                        return Err(crate::Error::InvalidValue(format!("{reference:?}")));
                    }
                    let kind = path.components[offset].child_name().unwrap();
                    serde_json::from_value::<BuildPhaseKind>(Value::String(kind.into()))
                        .map_err(|_| crate::Error::InvalidValue(kind.into()))?;
                }
                Ok(Self(reference))
            }
            pub fn reference(&self) -> &GroupTreeReference {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                Self::new(GroupTreeReference::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
phase_reference!(ProjectBuildPhaseReference, true);
phase_reference!(TargetBuildPhaseReference, false);
