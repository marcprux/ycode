// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

//! Read, inspect, and edit Xcode's JSON5 `project.xcproj` format.
//!
//! [`Project`] is the typed semantic model. [`ProjectDocument`] retains the source
//! and applies surgical edits; use it when opening existing projects in an IDE.
//!
//! ```
//! use ycode_project::{ProjectDocument, BuildSetting};
//! let source = r#"{files: [], 'default-configuration': 'Debug',
//!   localizations: {development: 'en'}, // keep this comment
//! }"#;
//! let mut document = ProjectDocument::parse(source)?;
//! document.set_build_setting(None, "SWIFT_VERSION", BuildSetting::from("6.0"))?;
//! assert!(document.source().contains("// keep this comment"));
//! # Ok::<(), ycode_project::Error>(())
//! ```

mod document;
mod error;
pub mod model;
mod syntax;
mod validation;
pub use document::{ProjectDocument, TextEdit};
pub use error::{Error, Result};
pub use model::*;

/// Parse a JSON5 value with the same duplicate-key, finite-number and depth checks
/// as a project document. Useful for editor commands accepting typed values.
pub fn parse_json5_value(source: &str) -> Result<serde_json::Value> {
    syntax::parse(source).map(|(_, value)| value)
}
