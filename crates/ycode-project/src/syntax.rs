// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

//! Source span index built from json-five's tokenizer, never by matching text.
//! Keeping the source itself avoids round-trip AST block-comment span bugs.
use crate::{Error, Result};
use json_five::tokenize::{TokType, tokenize_rt_str};
use serde_json::Value;
use std::{collections::BTreeSet, ops::Range};

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub range: Range<usize>,
    pub children: Children,
}
#[derive(Debug, Clone)]
pub(crate) enum Children {
    Scalar(Value),
    Object(Vec<Member>),
    Array(Vec<Node>),
}
#[derive(Debug, Clone)]
pub(crate) struct Member {
    pub key: String,
    pub start: usize,
    pub value: Node,
}

pub(crate) fn parse(source: &str) -> Result<(Node, Value)> {
    let tokens = tokenize_rt_str(source).map_err(|e| Error::Syntax {
        message: e.message,
        offset: e.index,
        line: e.lineno,
        column: e.colno,
    })?;
    let mut depth = 0usize;
    for (_, kind, _) in &tokens.tok_spans {
        match kind {
            TokType::LeftBrace | TokType::LeftBracket => {
                depth += 1;
                if depth > 128 {
                    return Err(Error::DepthLimit);
                }
            }
            TokType::RightBrace | TokType::RightBracket => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    // Validate the complete grammar before indexing. This rejects partial parses.
    json_five::model_from_str(source).map_err(|e| Error::Syntax {
        message: e.message,
        offset: e.index,
        line: e.lineno,
        column: e.colno,
    })?;
    let significant: Vec<_> = tokens
        .tok_spans
        .into_iter()
        .filter(|(_, kind, _)| {
            !matches!(
                kind,
                TokType::Whitespace | TokType::LineComment | TokType::BlockComment | TokType::EOF
            )
        })
        .collect();
    let mut scanner = Scanner {
        source,
        tokens: &significant,
        cursor: 0,
    };
    let root = scanner.node()?;
    if scanner.cursor != significant.len() {
        return Err(Error::Internal);
    }
    let value = root.value();
    Ok((root, value))
}
struct Scanner<'a> {
    source: &'a str,
    tokens: &'a [(usize, TokType, usize)],
    cursor: usize,
}
impl Scanner<'_> {
    fn token(&self) -> Result<&(usize, TokType, usize)> {
        self.tokens.get(self.cursor).ok_or(Error::Internal)
    }
    fn node(&mut self) -> Result<Node> {
        let (start, kind, end) = self.token()?.clone();
        self.cursor += 1;
        let children = match kind {
            TokType::LeftBrace => {
                let mut members = Vec::new();
                let mut names = BTreeSet::new();
                while self.token()?.1 != TokType::RightBrace {
                    let (key_start, _, key_end) = self.token()?.clone();
                    let key_source = &self.source[key_start..key_end];
                    let quoted = key_source.starts_with(['\'', '"']);
                    let key = decode_string(if quoted {
                        &key_source[1..key_source.len() - 1]
                    } else {
                        key_source
                    })?;
                    if !names.insert(key.clone()) {
                        return Err(Error::DuplicateKey {
                            key,
                            offset: key_start,
                        });
                    }
                    self.cursor += 2; // key and colon, already grammar-checked
                    let value = self.node()?;
                    members.push(Member {
                        key,
                        start: key_start,
                        value,
                    });
                    if self.token()?.1 == TokType::Comma {
                        self.cursor += 1;
                    }
                }
                Children::Object(members)
            }
            TokType::LeftBracket => {
                let mut values = Vec::new();
                while self.token()?.1 != TokType::RightBracket {
                    values.push(self.node()?);
                    if self.token()?.1 == TokType::Comma {
                        self.cursor += 1;
                    }
                }
                Children::Array(values)
            }
            TokType::Minus | TokType::Plus => {
                let end = self.token()?.2;
                self.cursor += 1;
                return Ok(Node {
                    range: start..end,
                    children: Children::Scalar(scalar(&self.source[start..end])?),
                });
            }
            _ => {
                return Ok(Node {
                    range: start..end,
                    children: Children::Scalar(scalar(&self.source[start..end])?),
                });
            }
        };
        let end = self.token()?.2;
        self.cursor += 1;
        Ok(Node {
            range: start..end,
            children,
        })
    }
}

pub(crate) fn pointer(path: &str) -> Result<Vec<String>> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    let rest = path
        .strip_prefix('/')
        .ok_or_else(|| Error::InvalidPointer(path.into()))?;
    rest.split('/')
        .map(|part| {
            let mut result = String::new();
            let mut chars = part.chars();
            while let Some(c) = chars.next() {
                if c == '~' {
                    result.push(match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err(Error::InvalidPointer(path.into())),
                    });
                } else {
                    result.push(c);
                }
            }
            Ok(result)
        })
        .collect()
}
pub(crate) fn escape_pointer(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}
pub(crate) fn index(part: &str) -> Option<usize> {
    if part.is_empty()
        || (part.starts_with('0') && part != "0")
        || !part.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    part.parse().ok()
}
impl Node {
    fn value(&self) -> Value {
        match &self.children {
            Children::Scalar(value) => value.clone(),
            Children::Object(members) => Value::Object(
                members
                    .iter()
                    .map(|m| (m.key.clone(), m.value.value()))
                    .collect(),
            ),
            Children::Array(values) => Value::Array(values.iter().map(Node::value).collect()),
        }
    }
    pub fn at(&self, parts: &[String]) -> Option<&Self> {
        let Some((first, rest)) = parts.split_first() else {
            return Some(self);
        };
        let node = match &self.children {
            Children::Object(members) => &members.iter().find(|m| m.key == *first)?.value,
            Children::Array(values) => values.get(index(first)?)?,
            Children::Scalar(_) => return None,
        };
        node.at(rest)
    }
}

// json-five 0.3.1's serde adapter mishandles negative hex, line continuations,
// surrogate pairs and non-escape characters. Decode scalar tokens here while
// keeping its tokenizer and grammar parser as the source of syntax validation.
fn scalar(source: &str) -> Result<Value> {
    if source.starts_with(['\'', '"']) {
        return decode_string(&source[1..source.len() - 1]).map(Value::String);
    }
    match source {
        "null" => return Ok(Value::Null),
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    let tokens = json_five::tokenize::tokenize_str(source).map_err(|_| Error::Internal)?;
    let normalized: String = tokens
        .tok_spans
        .iter()
        .filter(|(_, t, _)| *t != TokType::EOF)
        .map(|(start, _, end)| &source[*start..*end])
        .collect();
    let unsigned = normalized.trim_start_matches(['+', '-']);
    let negative = normalized.starts_with('-');
    let integer = if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        Some(i128::from_str_radix(hex, 16))
    } else if unsigned.bytes().all(|c| c.is_ascii_digit()) {
        Some(unsigned.parse::<i128>())
    } else {
        None
    };
    if let Some(integer) = integer {
        let mut integer = integer.map_err(|_| Error::InvalidValue(source.into()))?;
        if negative {
            integer = -integer;
        }
        if integer < 0 {
            return i64::try_from(integer)
                .map(Value::from)
                .map_err(|_| Error::InvalidValue(source.into()));
        }
        return u64::try_from(integer)
            .map(Value::from)
            .map_err(|_| Error::InvalidValue(source.into()));
    }
    let number = normalized
        .parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .ok_or_else(|| Error::InvalidValue(source.into()))?;
    Ok(Value::Number(number))
}
fn decode_string(source: &str) -> Result<String> {
    let invalid = || Error::InvalidValue(source.into());
    let mut chars = source.chars().peekable();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next().ok_or_else(invalid)? {
            'b' => result.push('\u{8}'),
            'f' => result.push('\u{c}'),
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            'v' => result.push('\u{b}'),
            '0' if !chars.peek().is_some_and(char::is_ascii_digit) => result.push('\0'),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' | '\u{2028}' | '\u{2029}' => {}
            escape @ ('u' | 'x') => {
                let read_hex =
                    |chars: &mut std::iter::Peekable<std::str::Chars<'_>>, count| -> Result<u32> {
                        let mut value = 0;
                        for _ in 0..count {
                            value = value * 16
                                + chars
                                    .next()
                                    .and_then(|c| c.to_digit(16))
                                    .ok_or_else(invalid)?;
                        }
                        Ok(value)
                    };
                let mut value = read_hex(&mut chars, if escape == 'u' { 4 } else { 2 })?;
                if (0xd800..=0xdbff).contains(&value) {
                    if chars.next() != Some('\\') || chars.next() != Some('u') {
                        return Err(invalid());
                    }
                    let low = read_hex(&mut chars, 4)?;
                    if !(0xdc00..=0xdfff).contains(&low) {
                        return Err(invalid());
                    }
                    value = 0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00);
                }
                result.push(char::from_u32(value).ok_or_else(invalid)?);
            }
            c if c.is_ascii_digit() => return Err(invalid()),
            other => result.push(other),
        }
    }
    Ok(result)
}
