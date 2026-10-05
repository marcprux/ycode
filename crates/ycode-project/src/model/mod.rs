// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

//! Typed representations of Apple's project.xcproj schema.
//!
//! Wire keys use kebab-case. Compact encodings are accepted alongside their
//! expanded forms. Use `ProjectDocument` to retain original source formatting.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Unrecognized object fields retained by extensible model objects.
pub type ExtraFields = BTreeMap<String, Value>;
fn default_true() -> bool {
    true
}
fn is_true(value: &bool) -> bool {
    *value
}
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

macro_rules! default_tagged_enum {
    ($(#[$meta:meta])* pub enum $name:ident, $default:literal { $($variant:ident($ty:ty) => $tag:literal),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Serialize)]
        #[serde(tag = "kind")]
        pub enum $name { $(#[serde(rename = $tag)] $variant($ty)),* }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let mut value = Value::deserialize(deserializer)?;
                if let Some(object) = value.as_object_mut() {
                    object.entry("kind").or_insert_with(|| Value::String($default.into()));
                }
                #[derive(Deserialize)]
                #[serde(tag = "kind")]
                enum Wire { $(#[serde(rename = $tag)] $variant($ty)),* }
                let wire: Wire = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(match wire { $(Wire::$variant(v) => Self::$variant(v)),* })
            }
        }
    };
}

mod build;
mod membership;
mod packages;
mod paths;
mod project;
mod references;
mod targets;
mod values;
pub use build::*;
pub use membership::*;
pub use packages::*;
pub use paths::*;
pub use project::*;
pub use references::*;
pub use targets::*;
pub use values::*;
