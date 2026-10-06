// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

/// Structured failures for IDE integrations, with English display messages.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Syntax {
        message: String,
        offset: usize,
        line: usize,
        column: usize,
    },
    Schema(String),
    InvalidValue(String),
    InvalidPointer(String),
    MissingPointer(String),
    DuplicateKey {
        key: String,
        offset: usize,
    },
    DuplicateTarget(String),
    UnsupportedCapability(String),
    DepthLimit,
    TargetNotFound(String),
    ConcurrentChange,
    NotRegularFile,
    NoSourceFile,
    Io(io::Error),
    Internal,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (label, detail): (&str, Option<&dyn fmt::Display>) = match self {
            Self::Syntax {
                message,
                line,
                column,
                ..
            } => return write!(f, "Invalid JSON5 ({line}:{column}): {message}"),
            Self::Schema(s) => ("Invalid Xcode project", Some(s)),
            Self::InvalidValue(s) => ("Invalid value", Some(s)),
            Self::InvalidPointer(s) => ("Invalid JSON pointer", Some(s)),
            Self::MissingPointer(s) => ("No value at JSON pointer", Some(s)),
            Self::DuplicateKey { key, offset } => {
                return write!(f, "Duplicate object key ({offset}): {key}");
            }
            Self::DuplicateTarget(s) => ("Duplicate target name", Some(s)),
            Self::UnsupportedCapability(s) => ("Unsupported required capability", Some(s)),
            Self::DepthLimit => ("Document exceeds the maximum nesting depth of 128", None),
            Self::TargetNotFound(s) => ("Target not found", Some(s)),
            Self::ConcurrentChange => ("The file changed on disk; reload it before saving", None),
            Self::NoSourceFile => (
                "The document has no source file; use write_new or save through the editor",
                None,
            ),
            Self::NotRegularFile => ("Expected a regular file", None),
            Self::Io(e) => ("File operation failed", Some(e)),
            Self::Internal => ("Unable to preserve the document safely", None),
        };
        f.write_str(label)?;
        if let Some(detail) = detail {
            write!(f, ": {detail}")?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Io(e) = self {
            Some(e)
        } else {
            None
        }
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Schema(e.to_string())
    }
}
