// Rust adaptation of Apple's xcode-project-format schema.
// Copyright © 2026 Apple Inc. and the xcode-project-format project authors
// Modifications Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::{fmt, str::FromStr};

macro_rules! string_types {
    ($($name:ident),* $(,)?) => {$(
        #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);
        impl From<String> for $name { fn from(value: String) -> Self { Self(value) } }
        impl From<&str> for $name { fn from(value: &str) -> Self { Self(value.into()) } }
        impl AsRef<str> for $name { fn as_ref(&self) -> &str { &self.0 } }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
        }
    )*};
}
string_types!(
    ObjectId,
    ConfigurationName,
    FileTypeId,
    ProductTypeId,
    Language,
    Capability,
    LocalTargetReference,
    SwiftPackageName,
    PlatformFilter,
    AssetTag,
    FolderMemberId
);

/// A build setting is a string or a list of strings, never a JSON boolean/number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BuildSetting {
    String(String),
    Array(Vec<String>),
}
impl From<&str> for BuildSetting {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}
impl From<String> for BuildSetting {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// Script text, accepting either a string or an array joined by newlines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultilineText(pub String);
impl Serialize for MultilineText {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let lines: Vec<_> = self.0.split('\n').collect();
        if lines.len() == 1 || (lines.len() == 2 && lines[1].is_empty()) {
            self.0.serialize(serializer)
        } else {
            lines.serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for MultilineText {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(match BuildSetting::deserialize(d)? {
            BuildSetting::String(s) => s,
            BuildSetting::Array(lines) => lines.join("\n"),
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketingVersion {
    pub major: i64,
    pub minor: i64,
    pub update: i64,
}
impl FromStr for MarketingVersion {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').map(str::parse::<i64>).collect();
        if !(2..=3).contains(&parts.len()) || parts.iter().any(Result::is_err) {
            return Err(crate::Error::InvalidValue(s.into()));
        }
        Ok(Self {
            major: parts[0].as_ref().copied().unwrap(),
            minor: parts[1].as_ref().copied().unwrap(),
            update: parts.get(2).map(|p| *p.as_ref().unwrap()).unwrap_or(0),
        })
    }
}
impl fmt::Display for MarketingVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if self.update != 0 {
            write!(f, ".{}", self.update)?;
        }
        Ok(())
    }
}
impl Serialize for MarketingVersion {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for MarketingVersion {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// Foundation text encoding: a well-known name or a numeric raw value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextEncoding {
    Named(TextEncodingName),
    Raw(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEncodingName {
    #[serde(rename = "ascii")]
    Ascii,
    #[serde(rename = "nextstep")]
    Nextstep,
    #[serde(rename = "japanese-euc")]
    JapaneseEUC,
    #[serde(rename = "utf8")]
    Utf8,
    #[serde(rename = "iso-latin-1")]
    IsoLatin1,
    #[serde(rename = "symbol")]
    Symbol,
    #[serde(rename = "non-lossy-ascii")]
    NonLossyASCII,
    #[serde(rename = "shift-jis")]
    ShiftJIS,
    #[serde(rename = "iso-latin-2")]
    IsoLatin2,
    #[serde(rename = "unicode")]
    Unicode,
    #[serde(rename = "windows-code-page-1251")]
    WindowsCP1251,
    #[serde(rename = "windows-code-page-1252")]
    WindowsCP1252,
    #[serde(rename = "windows-code-page-1253")]
    WindowsCP1253,
    #[serde(rename = "windows-code-page-1254")]
    WindowsCP1254,
    #[serde(rename = "windows-code-page-1250")]
    WindowsCP1250,
    #[serde(rename = "iso-2022-jp")]
    Iso2022JP,
    #[serde(rename = "macos-roman")]
    MacOSRoman,
    #[serde(rename = "utf16")]
    Utf16,
    #[serde(rename = "utf16-big-endian")]
    Utf16BigEndian,
    #[serde(rename = "utf16-little-endian")]
    Utf16LittleEndian,
    #[serde(rename = "utf32")]
    Utf32,
    #[serde(rename = "utf32-big-endian")]
    Utf32BigEndian,
    #[serde(rename = "utf32-little-endian")]
    Utf32LittleEndian,
}
