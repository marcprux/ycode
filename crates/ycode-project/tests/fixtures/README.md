# Upstream compatibility corpus

`upstream.json` maps fixture filenames to their complete JSON5 source text.
It contains 634 encodings emitted by Apple's Swift library at revision
[`296395785f7544f27ef929006b79af57e67092d3`](https://github.com/apple/xcode-project-format/tree/296395785f7544f27ef929006b79af57e67092d3),
using `Sources/Tests/TestInstances.swift`. The corpus includes empty, populated,
and additional values; types with more than 128 combinations are sampled
at deterministic indices across the full corpus, retaining the first 32 cases.

These are test fixtures, not bundled application assets. Copyright © 2026 Apple
Inc. and the xcode-project-format project authors. Licensed under Apache-2.0
WITH Swift-exception; see the package's [LICENSE-APPLE](../../LICENSE-APPLE) and [NOTICE](../../NOTICE).
The adjacent `upstream.json.license` records this exception without changing the JSON data.

Regenerate from a local upstream checkout (requires Swift 6.1 or newer):

```sh
python3 tools/export-swift-fixtures.py /tmp/xcode-project-format
```

Check both directions, including equality of the original and decoded Swift values:

```sh
YCODE_ROUNDTRIP_DIR=/tmp/ycode-roundtrip cargo test -p ycode-project --test upstream
python3 tools/export-swift-fixtures.py /tmp/xcode-project-format /tmp/ycode-roundtrip
```

The script builds a temporary copy of the supplied checkout. It never modifies
that checkout. Normal `cargo test` uses the saved corpus and needs no Swift installation.
Swift inline wrappers for file paths and package locations use test adapters
around the corresponding Rust fields/enum payloads. All 634 fixtures are checked;
see the exhaustive dispatch in `tests/upstream.rs`.
