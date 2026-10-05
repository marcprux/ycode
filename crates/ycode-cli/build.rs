// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=locales");
    let mut catalogs = BTreeMap::<String, BTreeMap<String, String>>::new();
    for entry in fs::read_dir("locales").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let locale = path.file_stem().unwrap().to_str().unwrap().to_owned();
        assert!(locale.chars().all(|c| c.is_ascii_lowercase()));
        catalogs.insert(
            locale,
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap(),
        );
    }
    let english = &catalogs["en"];
    for catalog in catalogs.values() {
        assert!(catalog.keys().eq(english.keys()), "translation keys differ");
    }
    let mut output = String::from("// SPDX-License-Identifier: MPL-2.0\n\npub mod str {\n");
    output.push_str("fn locale() -> String {\n");
    output.push_str("[\"LC_ALL\", \"LC_MESSAGES\", \"LANG\"].into_iter().find_map(|key| std::env::var(key).ok().filter(|s| !s.is_empty())).unwrap_or_default().split(['_', '-', '.']).next().unwrap_or(\"\").to_ascii_lowercase()\n}\n");
    for (key, value) in english {
        assert!(key.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
        output.push_str(&format!(
            "pub fn {key}() -> &'static str {{ match locale().as_str() {{\n"
        ));
        for (locale, catalog) in &catalogs {
            if locale != "en" {
                output.push_str(&format!("{locale:?} => {:?},\n", catalog[key]));
            }
        }
        output.push_str(&format!("_ => {value:?},\n}} }}\n"));
    }
    output.push_str("}\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("resources.rs"),
        output,
    )
    .unwrap();
}
