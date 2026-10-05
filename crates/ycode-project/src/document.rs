// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use crate::{
    BuildSetting, Error, Project, Result,
    syntax::{self, Children, Node},
};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    ops::Range,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// A UTF-8 byte-range edit, suitable for conversion to an editor's coordinate system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range<usize>,
    pub replacement: String,
}

/// A typed project backed by its original JSON5 source and a token-derived span index.
/// All mutations are transactional: invalid edits leave the document untouched.
#[derive(Debug, Clone)]
pub struct ProjectDocument {
    source: String,
    root: Node,
    value: Value,
    project: Project,
    origin: Option<(PathBuf, String)>,
}
impl ProjectDocument {
    pub fn parse(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        let (root, value) = syntax::parse(&source)?;
        let project: Project = serde_json::from_value(value.clone())?;
        project.validate()?;
        Ok(Self {
            source,
            root,
            value,
            project,
            origin: None,
        })
    }

    /// Open a file or an .xcodeproj bundle. Symlinks resolve to their current target.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = project_path(path.as_ref());
        let path = fs::canonicalize(path)?;
        if !fs::metadata(&path)?.is_file() {
            return Err(Error::NotRegularFile);
        }
        let source = fs::read_to_string(&path)?;
        let mut document = Self::parse(source.clone())?;
        document.origin = Some((path, source));
        Ok(document)
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn project(&self) -> &Project {
        &self.project
    }
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub fn get(&self, pointer: &str) -> Result<&Value> {
        syntax::pointer(pointer)?;
        self.value
            .pointer(pointer)
            .ok_or_else(|| Error::MissingPointer(pointer.into()))
    }
    pub fn source_range(&self, pointer: &str) -> Result<Range<usize>> {
        self.root
            .at(&syntax::pointer(pointer)?)
            .map(|n| n.range.clone())
            .ok_or_else(|| Error::MissingPointer(pointer.into()))
    }

    /// Replace a value, add an object property, or append using an array's `/-` pointer.
    /// Existing equal values are byte-for-byte no-ops. Explicit subtree replacement
    /// replaces its internal comments as well; prefer edit_project for field-wise edits.
    pub fn set(&mut self, pointer: &str, value: Value) -> Result<()> {
        self.transaction(|document| document.set_raw(pointer, &value))
    }
    /// Remove a value and its separator, retaining surrounding comments and whitespace.
    pub fn remove(&mut self, pointer: &str) -> Result<()> {
        self.transaction(|document| document.remove_raw(pointer))
    }
    /// Edit the typed model and reconcile only semantic differences into the source.
    /// Equal-length arrays are reconciled by index; resizing replaces that array.
    pub fn edit_project(&mut self, edit: impl FnOnce(&mut Project)) -> Result<()> {
        let before = serde_json::to_value(&self.project)?;
        let mut project = self.project.clone();
        edit(&mut project);
        project.validate()?;
        let after = serde_json::to_value(project)?;
        self.transaction(|document| document.reconcile("", &before, &after))
    }
    /// Set an unconditional or condition-qualified build setting at project/target scope.
    pub fn set_build_setting(
        &mut self,
        target: Option<&str>,
        key: &str,
        value: BuildSetting,
    ) -> Result<()> {
        let prefix = self.settings_pointer(target)?;
        let value = serde_json::to_value(value)?;
        self.transaction(|document| {
            if document.value.pointer(&prefix).is_none() {
                document.set_raw(&prefix, &serde_json::json!({}))?;
            }
            document.set_raw(&format!("{prefix}/{}", syntax::escape_pointer(key)), &value)
        })
    }
    pub fn remove_build_setting(&mut self, target: Option<&str>, key: &str) -> Result<()> {
        self.remove(&format!(
            "{}/{}",
            self.settings_pointer(target)?,
            syntax::escape_pointer(key)
        ))
    }
    fn settings_pointer(&self, target: Option<&str>) -> Result<String> {
        match target {
            None => Ok("/build-settings".into()),
            Some(name) => self
                .project
                .targets
                .iter()
                .position(|t| t.name() == name)
                .map(|i| format!("/targets/{i}/build-settings"))
                .ok_or_else(|| Error::TargetNotFound(name.into())),
        }
    }
    fn transaction(&mut self, edit: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        let mut candidate = self.clone();
        edit(&mut candidate)?;
        candidate.project = serde_json::from_value(candidate.value.clone())?;
        candidate.project.validate()?;
        *self = candidate;
        Ok(())
    }
    fn apply(&mut self, mut edits: Vec<TextEdit>) -> Result<()> {
        edits.reverse(); // At the same offset, insert the new entry before its separator.
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
        let mut source = self.source.clone();
        for edit in edits {
            source.replace_range(edit.range, &edit.replacement);
        }
        let (root, value) = syntax::parse(&source)?;
        self.source = source;
        self.root = root;
        self.value = value;
        Ok(())
    }
    fn set_raw(&mut self, pointer: &str, value: &Value) -> Result<()> {
        let parts = syntax::pointer(pointer)?;
        if self.value.pointer(pointer) == Some(value) {
            return Ok(());
        }
        if let Some(node) = self.root.at(&parts) {
            let range = node.range.clone();
            let mut replacement = json_literal(value)?;
            // Keep an existing single-quoted string's quote convention.
            if self.source[range.clone()].starts_with('\'')
                && let Some(string) = value.as_str()
            {
                replacement = single_quoted(string);
            }
            return self.apply(vec![TextEdit { range, replacement }]);
        }
        let (last, parent_parts) = parts
            .split_last()
            .ok_or_else(|| Error::InvalidPointer(pointer.into()))?;
        let parent = self
            .root
            .at(parent_parts)
            .ok_or_else(|| Error::MissingPointer(pointer.into()))?;
        let raw = json_literal(value)?;
        let (entry, starts, last_end) = match &parent.children {
            Children::Object(members) => (
                format!("{}: {raw}", json_literal(&Value::String(last.clone()))?),
                members.first().map(|m| m.start),
                members.last().map(|m| m.value.range.end),
            ),
            Children::Array(values) if last == "-" || syntax::index(last) == Some(values.len()) => {
                (
                    raw,
                    values.first().map(|v| v.range.start),
                    values.last().map(|v| v.range.end),
                )
            }
            _ => return Err(Error::MissingPointer(pointer.into())),
        };
        let close = parent.range.end - 1;
        let trailing_comma = last_end.and_then(|end| comma_in(&self.source, end..close));
        let mut edits = Vec::new();
        if let Some(end) = last_end
            && trailing_comma.is_none()
        {
            edits.push(TextEdit {
                range: end..end,
                replacement: ",".into(),
            });
        }
        let multiline = self.source[parent.range.clone()].contains('\n');
        let replacement = if multiline {
            let newline = if self.source.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            let base = line_indent(&self.source, parent.range.start);
            let child = starts
                .map(|s| line_indent(&self.source, s))
                .filter(|s| s.len() > base.len())
                .unwrap_or_else(|| {
                    format!("{base}{}", if base.contains('\t') { "\t" } else { "  " })
                });
            // Insert before the close token, preserving the preceding trivia verbatim.
            let existing_indent =
                &self.source[self.source[..close].rfind('\n').map_or(0, |i| i + 1)..close];
            let lead = if existing_indent.chars().all(|c| c == ' ' || c == '\t') {
                child
                    .strip_prefix(existing_indent)
                    .unwrap_or(&child)
                    .to_string()
            } else {
                format!("{newline}{child}")
            };
            format!(
                "{lead}{entry}{}{newline}{base}",
                if trailing_comma.is_some() { "," } else { "" }
            )
        } else {
            format!(
                "{}{entry}{}",
                if last_end.is_some() { " " } else { "" },
                if trailing_comma.is_some() { "," } else { "" }
            )
        };
        edits.push(TextEdit {
            range: close..close,
            replacement,
        });
        self.apply(edits)
    }
    fn remove_raw(&mut self, pointer: &str) -> Result<()> {
        let parts = syntax::pointer(pointer)?;
        let (last, parent_parts) = parts
            .split_last()
            .ok_or_else(|| Error::InvalidPointer(pointer.into()))?;
        let parent = self
            .root
            .at(parent_parts)
            .ok_or_else(|| Error::MissingPointer(pointer.into()))?;
        let entries: Vec<(usize, Range<usize>)> = match &parent.children {
            Children::Object(members) => members
                .iter()
                .map(|m| (m.start, m.value.range.clone()))
                .collect(),
            Children::Array(values) => values
                .iter()
                .map(|n| (n.range.start, n.range.clone()))
                .collect(),
            _ => return Err(Error::MissingPointer(pointer.into())),
        };
        let index = match &parent.children {
            Children::Object(members) => members.iter().position(|m| m.key == *last),
            Children::Array(_) => syntax::index(last).filter(|i| *i < entries.len()),
            _ => None,
        }
        .ok_or_else(|| Error::MissingPointer(pointer.into()))?;
        let (start, range) = &entries[index];
        let next = entries.get(index + 1).map_or(parent.range.end - 1, |e| e.0);
        let mut edits = vec![TextEdit {
            range: *start..range.end,
            replacement: String::new(),
        }];
        if let Some(comma) = comma_in(&self.source, range.end..next) {
            edits.push(TextEdit {
                range: comma..comma + 1,
                replacement: String::new(),
            });
        } else if index > 0
            && let Some(comma) = comma_in(&self.source, entries[index - 1].1.end..*start)
        {
            edits.push(TextEdit {
                range: comma..comma + 1,
                replacement: String::new(),
            });
        }
        self.apply(edits)
    }
    fn reconcile(&mut self, pointer: &str, before: &Value, after: &Value) -> Result<()> {
        if before == after {
            return Ok(());
        }
        match (before, after) {
            (Value::Object(a), Value::Object(b))
                if self.value.pointer(pointer).is_some_and(Value::is_object) =>
            {
                for key in a.keys().filter(|k| !b.contains_key(*k)) {
                    let child = format!("{pointer}/{}", syntax::escape_pointer(key));
                    if self.value.pointer(&child).is_some() {
                        self.remove_raw(&child)?;
                    }
                }
                for (key, value) in b {
                    let child = format!("{pointer}/{}", syntax::escape_pointer(key));
                    if let Some(old) = a.get(key) {
                        self.reconcile(&child, old, value)?;
                    } else {
                        self.set_raw(&child, value)?;
                    }
                }
                Ok(())
            }
            (Value::Array(a), Value::Array(b))
                if a.len() == b.len()
                    && self.value.pointer(pointer).is_some_and(Value::is_array) =>
            {
                for (i, (old, new)) in a.iter().zip(b).enumerate() {
                    self.reconcile(&format!("{pointer}/{i}"), old, new)?;
                }
                Ok(())
            }
            _ => self.set_raw(pointer, after),
        }
    }

    /// Compute a single minimal enclosing UTF-8 edit relative to an editor snapshot.
    pub fn text_edit(&self, original: &str) -> Option<TextEdit> {
        if original == self.source {
            return None;
        }
        let mut prefix = original
            .bytes()
            .zip(self.source.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while !original.is_char_boundary(prefix) || !self.source.is_char_boundary(prefix) {
            prefix -= 1;
        }
        let mut suffix = original[prefix..]
            .bytes()
            .rev()
            .zip(self.source[prefix..].bytes().rev())
            .take_while(|(a, b)| a == b)
            .count();
        while !original.is_char_boundary(original.len() - suffix)
            || !self.source.is_char_boundary(self.source.len() - suffix)
        {
            suffix -= 1;
        }
        Some(TextEdit {
            range: prefix..original.len() - suffix,
            replacement: self.source[prefix..self.source.len() - suffix].into(),
        })
    }
    /// Atomically replace the opened file after checking its contents still match.
    /// Permission bits are retained. The comparison is optimistic, not a file lock.
    pub fn save(&mut self) -> Result<()> {
        let (path, original) = self.origin.as_ref().ok_or(Error::NoSourceFile)?;
        if !fs::symlink_metadata(path)?.file_type().is_file() {
            return Err(Error::NotRegularFile);
        }
        if fs::read_to_string(path)? != *original {
            return Err(Error::ConcurrentChange);
        }
        if self.source == *original {
            return Ok(());
        }
        let temp = TempFile::write(path, &self.source, Some(fs::metadata(path)?.permissions()))?;
        if fs::read_to_string(path)? != *original {
            return Err(Error::ConcurrentChange);
        }
        fs::rename(&temp.path, path)?;
        self.origin.as_mut().unwrap().1 = self.source.clone();
        Ok(())
    }
    /// Write a new output file without overwriting any existing destination.
    pub fn write_new(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = project_path(path.as_ref());
        let temp = TempFile::write(&path, &self.source, None)?;
        // Linking is atomic and refuses existing paths, unlike rename.
        fs::hard_link(&temp.path, path)?;
        Ok(())
    }
}
fn single_quoted(value: &str) -> String {
    let json = serde_json::to_string(value)
        .expect("string serialization cannot fail")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    let mut out = String::from("'");
    let mut chars = json[1..json.len() - 1].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = chars.next().unwrap();
                if next != '"' {
                    out.push('\\');
                }
                out.push(next);
            }
            '\'' => out.push_str("\\'"),
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}
fn comma_in(source: &str, range: Range<usize>) -> Option<usize> {
    json_five::tokenize::tokenize_rt_str(&source[range.clone()])
        .ok()?
        .tok_spans
        .into_iter()
        .find(|(_, kind, _)| *kind == json_five::tokenize::TokType::Comma)
        .map(|(offset, _, _)| range.start + offset)
}
fn line_indent(source: &str, offset: usize) -> String {
    let start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    source[start..offset]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}
fn project_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("project.xcproj")
    } else {
        path.into()
    }
}
struct TempFile {
    path: PathBuf,
}
impl TempFile {
    fn write(
        destination: &Path,
        source: &str,
        permissions: Option<fs::Permissions>,
    ) -> Result<Self> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        loop {
            let path = parent.join(format!(
                ".ycode-{}-{}.tmp",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            let mut file = match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                result => result?,
            };
            let temp = Self { path };
            file.write_all(source.as_bytes())?;
            if let Some(permissions) = permissions {
                file.set_permissions(permissions)?;
            }
            file.sync_all()?;
            return Ok(temp);
        }
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn json_literal(value: &Value) -> Result<String> {
    Ok(serde_json::to_string(value)?
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029"))
}
