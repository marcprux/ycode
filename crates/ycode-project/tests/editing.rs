// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use serde_json::json;
use ycode_project::{BuildSetting, Error, ProjectDocument};

// Synthetic projects exercise JSON5 trivia; they are not shipped application resources.
const SOURCE: &str = "/* project */\r\n{\r\n  files: [],\r\n  'default-configuration': 'Debug',\r\n  localizations: {development: 'en'},\r\n  targets: [{name: 'App', id: 'ABC', 'build-settings': {SWIFT_VERSION: '5.0',},}],\r\n  'build-settings': {\r\n    // Keep the explanation\r\n    'SWIFT_VERSION': '5.0', /* keep block */\r\n    'FLAGS[sdk=iphoneos*]': ['a', 'b'],\r\n  },\r\n  future: {hex: 0x2a, fraction: +.5, escaped: 'a\\\'b'},\r\n}\r\n// end\r\n";

#[test]
fn untouched_source_and_equal_updates_are_byte_identical() {
    let mut doc = ProjectDocument::parse(SOURCE).unwrap();
    assert_eq!(doc.source(), SOURCE);
    doc.set_build_setting(None, "SWIFT_VERSION", "5.0".into())
        .unwrap();
    doc.edit_project(|_| {}).unwrap();
    assert_eq!(doc.source(), SOURCE);
}
#[test]
fn changing_one_setting_only_changes_its_token() {
    let mut doc = ProjectDocument::parse(SOURCE).unwrap();
    doc.set_build_setting(None, "SWIFT_VERSION", "6.0".into())
        .unwrap();
    assert_eq!(
        doc.source(),
        SOURCE.replace("'SWIFT_VERSION': '5.0'", "'SWIFT_VERSION': '6.0'")
    );
    assert_eq!(
        doc.project().build_settings["SWIFT_VERSION"],
        BuildSetting::from("6.0")
    );
}
#[test]
fn typed_edit_does_not_materialize_defaults_or_change_compact_forms() {
    let source = "{files: [], 'default-configuration':'Debug', localizations:{development:'en'}, targets:[{id:'A',name:'App','build-phases':['compile-sources']}]}";
    let mut doc = ProjectDocument::parse(source).unwrap();
    doc.edit_project(|p| p.targets[0].common_mut().name = "Tool".into())
        .unwrap();
    assert_eq!(doc.source(), source.replace("'App'", "'Tool'"));
}
#[test]
fn inserted_removed_properties_keep_comments_and_are_parseable() {
    for settings in [
        "{}",
        "{A:'a'}",
        "{A:'a',}",
        "{\n  A: 'a' // trailing\n}",
        "{/*empty*/}",
        "{A:'a'/*block*/}",
    ] {
        let source = format!(
            "{{files:[], 'default-configuration':'Debug',localizations:{{development:'en'}},'build-settings':{settings}}}"
        );
        let mut doc = ProjectDocument::parse(&source).unwrap();
        doc.set_build_setting(None, "B", "b".into()).unwrap();
        assert_eq!(doc.project().build_settings["B"], BuildSetting::from("b"));
        if settings.contains("// trailing") {
            assert!(doc.source().contains("// trailing"));
        }
        if settings.contains("/*block*/") {
            assert!(doc.source().contains("/*block*/"));
        }
        doc.remove_build_setting(None, "B").unwrap();
        assert!(!doc.project().build_settings.contains_key("B"));
        ProjectDocument::parse(doc.source()).unwrap();
    }
}
#[test]
fn removal_handles_first_middle_last_and_only_members() {
    for field in ["A", "B", "C"] {
        for suffix in ["", ","] {
            let source = format!(
                "{{files:[], 'default-configuration':'D',localizations:{{development:'en'}},extra:{{A:1/*a*/,B:2/*b*/,C:3/*c*/{suffix}}}}}"
            );
            let mut doc = ProjectDocument::parse(&source).unwrap();
            doc.remove(&format!("/extra/{field}")).unwrap();
            for comment in ["/*a*/", "/*b*/", "/*c*/"] {
                assert!(doc.source().contains(comment));
            }
            assert_eq!(doc.get("/extra").unwrap().as_object().unwrap().len(), 2);
        }
    }
}
#[test]
fn array_append_and_remove_preserve_trivia() {
    let source = "{files:[], 'default-configuration':'D',localizations:{development:'en'},extra:[1, /*between*/ 2, //last\n]}";
    let mut doc = ProjectDocument::parse(source).unwrap();
    doc.set("/extra/-", json!(3)).unwrap();
    doc.remove("/extra/1").unwrap();
    assert_eq!(doc.get("/extra").unwrap(), &json!([1, 3]));
    assert!(doc.source().contains("/*between*/"));
    assert!(doc.source().contains("//last"));
}
#[test]
fn pointer_escapes_unicode_and_string_escaping() {
    let mut doc = ProjectDocument::parse(SOURCE).unwrap();
    doc.set("/future/a~1b~0c", json!("é 🦀")).unwrap();
    assert_eq!(doc.get("/future/a~1b~0c").unwrap(), &json!("é 🦀"));
    for value in ["'", "\\", "\\'", "\"", "é\n🦀", "\u{2028}"] {
        doc.set("/future/escaped", json!(value)).unwrap();
        assert_eq!(doc.get("/future/escaped").unwrap(), &json!(value));
    }
    for pointer in ["future", "/future/~2", "/files/01", "/files/+0"] {
        assert!(doc.set(pointer, json!(true)).is_err(), "{pointer}");
    }
}
#[test]
fn invalid_mutation_rolls_back_and_unsupported_capabilities_are_rejected() {
    let mut doc = ProjectDocument::parse(SOURCE).unwrap();
    assert!(doc.set("/targets/0/name", json!(123)).is_err());
    assert!(doc.remove("/localizations").is_err());
    assert!(
        doc.set("/required-capabilities", json!(["future feature"]))
            .is_err()
    );
    assert_eq!(doc.source(), SOURCE);
    assert!(
        doc.set_build_setting(Some("missing"), "A", "b".into())
            .is_err()
    );
}
#[test]
fn duplicates_escaped_duplicates_nonfinite_and_excessive_nesting_are_rejected() {
    for source in [
        "{a:1,a:2}",
        "{a:1,'\\u0061':2}",
        "{a:NaN}",
        "{a:-Infinity}",
        "{} {}",
        "{a:}",
    ] {
        assert!(
            ycode_project::parse_json5_value(source).is_err(),
            "{source}"
        );
    }
    let deep = format!("{}0{}", "[".repeat(129), "]".repeat(129));
    assert!(matches!(
        ycode_project::parse_json5_value(&deep),
        Err(Error::DepthLimit)
    ));
}
#[test]
fn text_edits_have_utf8_boundaries_and_reproduce_the_document() {
    let mut doc = ProjectDocument::parse(SOURCE).unwrap();
    doc.set("/future/escaped", json!("é🦀")).unwrap();
    let before = doc.source().to_string();
    doc.set("/future/escaped", json!("ê🦁")).unwrap();
    let edit = doc.text_edit(&before).unwrap();
    let mut applied = before;
    applied.replace_range(edit.range, &edit.replacement);
    assert_eq!(applied, doc.source());
}

#[test]
fn json5_scalars_follow_spec_despite_upstream_adapter_bugs() {
    for (source, expected) in [
        ("-0x2a", json!(-42)),
        ("- /*sign*/ 0X2A", json!(-42)),
        ("-9223372036854775808", json!(i64::MIN)),
        ("'line\\\ncontinuation'", json!("linecontinuation")),
        ("'line\\\r\ncontinuation'", json!("linecontinuation")),
        (r"'\a\q\x41'", json!("aqA")),
        (r"'\ud83e\udd80'", json!("🦀")),
    ] {
        assert_eq!(
            ycode_project::parse_json5_value(source).unwrap(),
            expected,
            "{source}"
        );
    }
    for source in [
        "1e999",
        "-1e999",
        "NaN",
        r"'\01'",
        r"'\ud800'",
        "18446744073709551616",
    ] {
        assert!(
            ycode_project::parse_json5_value(source).is_err(),
            "{source}"
        );
    }
}
