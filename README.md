<!-- SPDX-License-Identifier: MPL-2.0 -->

# ycode

[![CI](https://github.com/marcprux/ycode/actions/workflows/ci.yml/badge.svg)](https://github.com/marcprux/ycode/actions/workflows/ci.yml)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](https://github.com/marcprux/ycode/blob/main/LICENSE)

**Read and edit Xcode’s JSON5 projects from Rust—without rewriting the whole file.**

`ycode` is a Rust workspace for tools that work with Xcode’s `project.xcproj`
format. It provides a typed project model, a source-preserving editor, and a CLI
for inspecting projects and making focused changes.

The model is an idiomatic Rust adaptation of
[Apple’s xcode-project-format](https://github.com/apple/xcode-project-format).
It is intended for IDE integrations, project tools, automation, and an eventual
build driver.

| Crate | Purpose |
| --- | --- |
| `ycode-project` | Serde models, JSON5 parsing, transactional edits, and file persistence |
| `ycode-cli` | The `ycode` executable and its extensible subcommand tree |

## Why ycode?

- **Typed access:** projects, native/aggregate/external targets, configurations,
  file references, synchronized folders, groups, build phases, build rules,
  membership exceptions, Swift packages, dependencies, and reference paths.
- **Small diffs:** no-op edits preserve every byte; changing an existing scalar
  touches only its value token. Comments, key order, whitespace, CRLF, trailing
  commas, and untouched number/string spellings remain intact.
- **Editor-friendly:** structured errors, UTF-8 source ranges, text edits,
  transactional validation, and no process-wide mutable project state.
- **Careful writes:** preview edits first, detect external changes before saving,
  replace files atomically, and refuse to overwrite a separate output file.
- **A small dependency footprint:** the library uses `serde`, `serde_json`, and
  `json-five`; the CLI adds `clap`. No async runtime or platform SDK is required.

## Get started

Requires **Rust 1.89+** (edition 2024). Build and install from this checkout:

```sh
cargo build --workspace
cargo install --path crates/ycode-cli --locked

ycode --help
ycode project -p examples/Example.xcodeproj info
```

The package names are `ycode-project` and `ycode-cli`; the executable is `ycode`.
For a local library dependency:

```toml
[dependencies]
ycode-project = { path = "../ycode/crates/ycode-project" }
```

The initial release is prepared as version `0.1.0`. Publishing to crates.io is
separate from building or installing this checkout.

## Command line

Project operations live under `ycode project`, leaving room for future commands
such as `ycode build`. Pass either a `project.xcproj` file or a directory containing
one. Without `--project`, the CLI reads `./project.xcproj`.

```sh
# Summaries and structured information.
ycode project -p Example.xcodeproj info
ycode project -p Example.xcodeproj targets --json
ycode project -p Example.xcodeproj files
ycode project -p Example.xcodeproj packages
ycode project -p Example.xcodeproj configurations
ycode project -p Example.xcodeproj validate

# Extract any value with an RFC 6901 JSON pointer.
ycode project -p Example.xcodeproj get /targets/0/build-settings

# Preview a targeted setting change; the input file stays untouched.
ycode project -p Example.xcodeproj settings --target Example set SWIFT_VERSION 6.0

# Apply the same change to the input file.
ycode project -p Example.xcodeproj settings --target Example set SWIFT_VERSION 6.0 --write

# Set a string, or a JSON5 value such as a list.
ycode project -p Example.xcodeproj set /organization 'Example Studio' --string --write
ycode project -p Example.xcodeproj set '/targets/0/build-settings/OTHER_SWIFT_FLAGS' \
  "['\$(inherited)', '-DDEBUG']" --output updated.xcproj

# Remove a setting or another optional field.
ycode project -p Example.xcodeproj settings --target Example remove SWIFT_VERSION --write
ycode project -p Example.xcodeproj remove /organization --write

# Explicitly generate formatted JSON from the semantic model.
ycode project -p Example.xcodeproj export > exported.json
```

`info`, `targets`, and `validate` offer human-readable output or `--json`.
`files`, `packages`, `configurations`, `get`, and `settings list` emit JSON.
Mutation commands print the resulting **JSON5 source** unless `--write` or
`--output` is supplied; these two options are mutually exclusive. Errors go to
stderr and return a nonzero exit status.

Pointers escape `~` as `~0` and `/` as `~1`. `set` can replace a value, add a
property to an existing object, or append to an array with `/-`. Intermediate
containers must already exist. `settings set` creates a missing build-settings
object and treats its value as a literal string. Use `set` for string arrays.

Build-setting keys, including suffixes such as `[config=Debug]` or
`[sdk=iphoneos*]`, are preserved exactly. The tool does not evaluate them.

## Library

### Read the typed model

```rust
use ycode_project::ProjectDocument;

let document = ProjectDocument::open("Example.xcodeproj")?;
for target in &document.project().targets {
    println!("{}: {:?}", target.name(), target.common().product_type_id());
}
```

Rust types use `snake_case` fields, enums for alternatives, newtypes for IDs and
names, and deterministic maps/sets. Serde handles Apple’s kebab-case keys,
compact encodings, and omitted defaults. Configuration names, name paths,
project membership, and build phases accept the compact forms used by Xcode.

### Edit while preserving source

```rust
use ycode_project::{BuildSetting, ProjectDocument};

let mut document = ProjectDocument::open("Example.xcodeproj")?;
document.set_build_setting(
    Some("Example"),
    "SWIFT_VERSION",
    BuildSetting::from("6.0"),
)?;
document.save()?;
```

For an IDE, parse an in-memory buffer and apply the returned text edit through
the editor’s own undo/save system:

```rust
use ycode_project::ProjectDocument;

let buffer = "{files: [], 'default-configuration': 'Debug', \
              localizations: {development: 'en'}}";
let mut document = ProjectDocument::parse(buffer)?;
document.edit_project(|project| {
    project.organization = Some("Example Studio".into());
})?;

if let Some(edit) = document.text_edit(buffer) {
    // Ranges are UTF-8 byte offsets. Convert to UTF-16 positions for LSP clients.
    let mut updated = buffer.to_owned();
    updated.replace_range(edit.range, &edit.replacement);
    assert_eq!(updated, document.source());
}
```

`source_range(pointer)` locates a value in the current source.
`edit_project` compares the model before and after your closure and applies only
changed fields. An invalid edit leaves the original document unchanged.
`Project::new`, `Project::from_json5`, and `Project::to_json_pretty` support tools
that only need the semantic model or generate new projects.

### Preservation contract

| Operation | Behavior |
| --- | --- |
| Parse, inspect, or set an equal value | Source remains byte-for-byte identical |
| Change an existing scalar | Replaces its value token; preserves surrounding trivia and string quote style |
| Add/remove a property or array entry | Adjusts the member and necessary separators; retains other source text |
| Edit typed object fields | Reconciles differences without materializing unrelated defaults |
| Edit an equal-length array through the model | Reconciles by index |
| Explicitly replace a subtree or resize a typed array | Replaces that subtree, including its internal comments |
| Export/generate semantic JSON | Normalizes formatting and may expand compact forms |

Unknown fields stay in the document source, and extensible model structs expose
`extra` maps. Keep reserved schema keys out of those maps. Unknown enum kinds are
rejected; unsupported `required-capabilities` also prevent validated edits.

`save` retains permission bits and checks the last-loaded contents before an
atomic replacement. This is optimistic conflict detection, not a cross-process
lock: there is a small check-to-rename race. Symlinks resolve when opened and are
not replaced themselves. Other filesystem metadata, such as extended attributes,
is not copied. `write_new` uses a same-directory temporary file and an atomic
hard link, so it requires a filesystem supporting hard links.

## Compatibility and scope

The initial model targets Apple’s schema at
[`296395785f7544f27ef929006b79af57e67092d3`](https://github.com/apple/xcode-project-format/tree/296395785f7544f27ef929006b79af57e67092d3).
Compatibility tests use a saved corpus of 634 Swift-generated encodings. All
634 Rust outputs have also been decoded by Apple’s Swift library and
compared for semantic equality with the original Swift values.

This is an initial implementation, not an Xcode-certified replacement. It supports
`project.xcproj`, not the legacy `project.pbxproj` format. It does not build,
resolve filesystem references or package dependencies, expand synchronized
folders, evaluate build settings, or edit schemes/workspaces. Xcode remains the
final authority for loading and building a complete project.

JSON5 parsing rejects duplicate object keys, non-finite numbers, integers outside
`i64`/`u64`, unpaired Unicode surrogates, and nesting deeper than 128 containers.
The document layer compensates for scalar-decoding issues in `json-five` 0.3.1,
and edits source spans directly rather than reprinting its round-trip AST.

Dependencies deliberately match `day-cli`’s compatible version ranges:
`serde = "1"`, `serde_json = "1"`, `json-five = "=0.3.1"`, and
`clap = "4.6"` with `derive`/`wrap_help`. The library does not depend on the CLI.

App-owned help and messages use generated `res::str` accessors from checked
English/French catalogs. `LC_ALL`, `LC_MESSAGES`, then `LANG` select the language;
unsupported locales use English. Protocol keys, project content, and dependency
or operating-system diagnostics are not translated.

## Development

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --all-features --locked
cargo doc --workspace --all-features --no-deps --locked
```

[GitHub Actions](https://github.com/marcprux/ycode/actions/workflows/ci.yml) runs
on every push and pull request, and can also be started manually. The full test
suite and doc tests run on **macOS, Ubuntu, and Windows**, using both stable Rust
and the minimum supported version, **Rust 1.89.0**. This includes CLI integration,
source-preserving edits, filesystem behavior, model serialization, and all 634
saved upstream compatibility fixtures. Unix permission and symlink assertions
run on macOS and Ubuntu. A fast initial job requires clean `cargo fmt` and
Clippy results, treating every Clippy warning as an error. Only after that job
passes do the three OS runners start the full suite. Each OS also builds the
documentation with warnings treated as errors.

Cargo.lock is checked in for reproducible CLI and CI builds. LF checkout rules
keep source and fixtures consistent across platforms. Dependabot checks Cargo
dependencies and GitHub Actions weekly; dependency changes must pass the same CI.

Normal tests require no Swift toolchain or network access once Cargo dependencies
are available. To regenerate the corpus or check Rust output with Swift, see the
[fixture instructions](https://github.com/marcprux/ycode/blob/main/crates/ycode-project/tests/fixtures/README.md).
The CLI dispatch separates top-level commands from project operations so future
build and IDE integrations can reuse `ycode-project` independently.

## License

ycode’s Rust implementation, CLI, documentation, and original examples are
licensed under the [Mozilla Public License 2.0 (MPL-2.0)](https://github.com/marcprux/ycode/blob/main/LICENSE).
Both crates declare this license and their source files identify it
with SPDX headers.

The schema is adapted from Apple’s `xcode-project-format`. Apple’s copyright
and attribution are retained in the adapted files and
[NOTICE](https://github.com/marcprux/ycode/blob/main/NOTICE). The saved Apple-derived
test corpus retains **Apache-2.0 WITH Swift-exception**; the original license is
included in [LICENSE-APPLE](https://github.com/marcprux/ycode/blob/main/LICENSE-APPLE)
and both crate packages. The [fixture provenance](https://github.com/marcprux/ycode/blob/main/crates/ycode-project/tests/fixtures/README.md)
and its adjacent SPDX license file identify this exception. ycode is an independent
project.
