#!/usr/bin/env python3
# Copyright © 2026 ycode contributors

"""Export Apple's test values using a disposable copy of its Swift package.

Usage: python3 tools/export-swift-fixtures.py /path/to/xcode-project-format
Append a Rust round-trip output directory to verify semantic equality in Swift.
Requires Swift 6.1+. Never modifies the supplied checkout.
"""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
upstream = Path(sys.argv[1]).resolve()
types = list(dict.fromkeys(re.findall(r"test\((XCSchema\.[\w.]+)\.self\)",
    (upstream / "Sources/Tests/RoundTripSomeOfEverythingTests.swift").read_text())))
with tempfile.TemporaryDirectory(prefix="ycode-swift-") as directory:
    work = Path(directory)
    output = work / "fixtures"
    output.mkdir()
    package = work / "package"
    shutil.copytree(upstream, package, ignore=shutil.ignore_patterns(".git", ".build"))
    exporter = '''import Foundation
import Testing
import XcodeProjectFormat
struct YcodeFixtureExportTests {
    @Test func exportFixtures() throws {
        let root = URL(fileURLWithPath: ProcessInfo.processInfo.environment["YCODE_FIXTURE_DIR"]!)
        func export<T: TestInstanceDefining & XCJSON.Codable & Equatable>(_ type: T.Type, _ name: String) throws {
            let values = [T.emptyTestValue, T.populatedTestValue] + T.additionalTestValues
            // NamePath's combinatorial corpus has over 100,000 cases. Keep a
            // deterministic sample spanning it, plus the first 32 edge cases.
            let indices: [Int] = values.count <= 128 ? Array(values.indices)
                : Array(0..<32) + (0..<96).map { 32 + $0 * (values.count - 33) / 95 }
            for index in indices {
                let value = values[index]
                let data = try XCJSON.Encoder.data(for: value, options: .defaultOptions)
                let filename = "\\(name)-\\(index).json5"
                if let roundtrip = ProcessInfo.processInfo.environment["YCODE_SWIFT_VERIFY_DIR"] {
                    let path = URL(fileURLWithPath: roundtrip).appendingPathComponent(filename)
                    if FileManager.default.fileExists(atPath: path.path) {
                        do {
                            let decoded: T = try XCJSON.Decoder.decode(data: Data(contentsOf: path))
                            #expect(decoded == value, "\\(filename)")
                        } catch {
                            Issue.record("\\(filename): \\(error)")
                        }
                    }
                } else {
                    try data.write(to: root.appendingPathComponent(filename))
                }
            }
        }
'''
    exporter += "\n".join(f'        try export({t}.self, "{t.removeprefix("XCSchema.")}")' for t in types)
    exporter += "\n    }\n}\n"
    (package / "Sources/Tests/YcodeFixtureExportTests.swift").write_text(exporter)
    env = dict(os.environ, YCODE_FIXTURE_DIR=str(output),
        CLANG_MODULE_CACHE_PATH=str(work / "clang-cache"),
        SWIFTPM_MODULECACHE_OVERRIDE=str(work / "swift-cache"))
    if len(sys.argv) > 2:
        env["YCODE_SWIFT_VERIFY_DIR"] = str(Path(sys.argv[2]).resolve())
    subprocess.run(["swift", "test", "--package-path", str(package), "--disable-sandbox",
        "--scratch-path", str(work / "build"), "--filter", "YcodeFixtureExportTests"], env=env, check=True)
    if len(sys.argv) == 2:
        corpus = {p.name: p.read_text() for p in sorted(output.glob("*.json5"))}
        (root / "crates/ycode-project/tests/fixtures/upstream.json").write_text(
            json.dumps(corpus, ensure_ascii=False, indent=2) + "\n")
