// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use crate::res;
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

/// Structured failures suitable for localization in an IDE.
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
            } => return write!(f, "{} ({line}:{column}): {message}", res::str::syntax()),
            Self::Schema(s) => (res::str::schema(), Some(s)),
            Self::InvalidValue(s) => (res::str::invalid_value(), Some(s)),
            Self::InvalidPointer(s) => (res::str::invalid_pointer(), Some(s)),
            Self::MissingPointer(s) => (res::str::missing_pointer(), Some(s)),
            Self::DuplicateKey { key, offset } => {
                return write!(f, "{} ({offset}): {key}", res::str::duplicate_key());
            }
            Self::DuplicateTarget(s) => (res::str::duplicate_target(), Some(s)),
            Self::UnsupportedCapability(s) => (res::str::unsupported_capability(), Some(s)),
            Self::DepthLimit => (res::str::depth_limit(), None),
            Self::TargetNotFound(s) => (res::str::target_not_found(), Some(s)),
            Self::ConcurrentChange => (res::str::concurrent_change(), None),
            Self::NoSourceFile => (res::str::no_source_file(), None),
            Self::NotRegularFile => (res::str::not_regular(), None),
            Self::Io(e) => (res::str::io(), Some(e)),
            Self::Internal => (res::str::internal(), None),
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
