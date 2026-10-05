// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use serde_json::json;
use ycode_project::*;

#[test]
fn model_requires_upstream_required_fields_and_unique_target_names() {
    for source in [
        "{}",
        "{files:[],localizations:{development:'en'}}",
        "{files:[],'default-configuration':'D',localizations:{development:'en'},targets:[{name:'A'}]}",
        "{files:[],'default-configuration':'D',localizations:{development:'en'},targets:[{name:'A',id:'1'},{name:'A',id:'2'}]}",
        "{files:[],'default-configuration':'D',localizations:{development:'en'},'build-settings':{FLAG:true}}",
    ] {
        assert!(Project::from_json5(source).is_err(), "{source}");
    }
}
#[test]
fn absent_and_explicit_null_products_groups_are_distinct() {
    let base = "{files:[],'default-configuration':'D',localizations:{development:'en'}}";
    assert!(Project::from_json5(base).unwrap().products_group.is_some());
    let mut document = ProjectDocument::parse(base).unwrap();
    document.set("/products-group", json!(null)).unwrap();
    assert!(document.project().products_group.is_none());
    let generated = document.project().to_json_pretty().unwrap();
    assert!(
        Project::from_json5(&generated)
            .unwrap()
            .products_group
            .is_none()
    );
}
#[test]
fn unusual_names_use_unambiguous_component_arrays() {
    let path = GroupTreeReference::NamePath(NamePath {
        components: vec![
            NamePathComponent::Child("id:literal".into()),
            NamePathComponent::Child("a/b".into()),
            NamePathComponent::Child("..".into()),
        ],
    });
    let encoded = serde_json::to_value(&path).unwrap();
    assert_eq!(encoded, json!(["id:literal",{"name":"a/b"},{"name":".."}]));
    assert_eq!(
        serde_json::from_value::<GroupTreeReference>(encoded).unwrap(),
        path
    );
    let reference = ProjectBuildPhaseReference::new(GroupTreeReference::NamePath(NamePath {
        components: vec![
            NamePathComponent::Child("a/b".into()),
            NamePathComponent::Child("compile-sources".into()),
        ],
    }))
    .unwrap();
    let encoded = serde_json::to_value(ProjectBuildFile::Compact(reference)).unwrap();
    assert_eq!(
        encoded,
        json!({"build-phase":[{"name":"a/b"},"compile-sources"]})
    );
}
#[test]
fn unknown_nested_properties_survive_typed_changes_without_duplicate_fields() {
    let source = r#"{files: [], 'default-configuration':'D', localizations:{development:'en'},
      packages:[{kind:'remote',repository:'https://example.test/package',future:true}],
      targets:[{id:'a',name:'App','build-phases':[{kind:'script',shell:'/bin/sh',script:'true',future:{a:1}}],future:42}],future:'keep'}"#;
    let mut document = ProjectDocument::parse(source).unwrap();
    document
        .edit_project(|project| project.targets[0].common_mut().name = "Renamed".into())
        .unwrap();
    assert_eq!(document.source(), source.replace("'App'", "'Renamed'"));
    let exported = document.project().to_json_pretty().unwrap();
    let reparsed = ProjectDocument::parse(exported).unwrap();
    assert_eq!(reparsed.get("/packages/0/future").unwrap(), &json!(true));
    assert_eq!(
        reparsed.get("/targets/0/build-phases/0/future").unwrap(),
        &json!({"a":1})
    );
}
