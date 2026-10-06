// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

// Synthetic project fixture, not an application resource.
const SOURCE: &str = "// project comment\n{files:[], 'default-configuration':'Debug',localizations:{development:'en'},targets:[{name:'App',id:'A'}]}\n";
struct Fixture {
    directory: PathBuf,
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "ycode-cli-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("project.xcproj");
        fs::write(&path, SOURCE).unwrap();
        Self { directory, path }
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ycode"))
            .arg("project")
            .arg("-p")
            .arg(&self.path)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
fn successful(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
#[test]
fn inspect_get_and_validate() {
    let f = Fixture::new();
    let info = successful(f.run(&["info", "--json"]));
    let info: serde_json::Value = serde_json::from_str(&info).unwrap();
    assert_eq!(info["targets"], 1);
    assert_eq!(
        successful(f.run(&["info"])),
        "Targets: 1\nFile-tree entries: 0\nPackages: 0\nConfigurations: 0\nDefault configuration: Debug\nDevelopment language: en\n"
    );
    assert!(successful(f.run(&["targets"])).contains("App"));
    assert_eq!(
        successful(f.run(&["get", "/targets/0/name"])).trim(),
        "\"App\""
    );
    assert_eq!(successful(f.run(&["validate"])).trim(), "Project is valid");
    assert!(!f.run(&["get", "/missing"]).status.success());
}
#[test]
fn mutations_preview_by_default_then_write_or_create_output() {
    let f = Fixture::new();
    let args = ["settings", "--target", "App", "set", "SWIFT_VERSION", "6.0"];
    let preview = successful(f.run(&args));
    assert!(preview.contains("6.0"));
    assert!(preview.starts_with("// project comment\n"));
    assert_eq!(fs::read_to_string(&f.path).unwrap(), SOURCE);
    let mut write = args.to_vec();
    write.push("--write");
    successful(f.run(&write));
    assert_eq!(fs::read_to_string(&f.path).unwrap(), preview);
    let out = f.directory.join("copy.xcproj");
    successful(f.run(&[
        "set",
        "/organization",
        "Example",
        "--string",
        "--output",
        out.to_str().unwrap(),
    ]));
    assert!(fs::read_to_string(&out).unwrap().contains("Example"));
    assert!(
        !f.run(&[
            "set",
            "/organization",
            "Overwrite",
            "--string",
            "--output",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(fs::read_to_string(&out).unwrap().contains("Example"));
    assert!(
        !f.run(&[
            "set",
            "/organization",
            "x",
            "--string",
            "--write",
            "--output",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
}
#[test]
fn invalid_edit_never_touches_input_and_errors_have_nonzero_status() {
    let f = Fixture::new();
    assert!(
        !f.run(&["set", "/targets/0/name", "3", "--write"])
            .status
            .success()
    );
    assert!(!f.run(&["remove", "/files", "--write"]).status.success());
    assert!(
        !f.run(&[
            "settings", "--target", "missing", "set", "A", "b", "--write"
        ])
        .status
        .success()
    );
    assert_eq!(fs::read_to_string(&f.path).unwrap(), SOURCE);
}
#[test]
fn bundles_work_and_messages_remain_english_in_other_locales() {
    let f = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_ycode"))
        .env("LC_ALL", "fr_FR.UTF-8")
        .args(["project", "-p"])
        .arg(&f.directory)
        .arg("validate")
        .output()
        .unwrap();
    assert_eq!(successful(output).trim(), "Project is valid");
    let output = Command::new(env!("CARGO_BIN_EXE_ycode"))
        .env("LC_ALL", "fr_FR.UTF-8")
        .args(["project", "--help"])
        .output()
        .unwrap();
    assert!(successful(output).contains("Read and update project.xcproj files"));
    let output = Command::new(env!("CARGO_BIN_EXE_ycode"))
        .env("LC_ALL", "fr_FR.UTF-8")
        .args(["project", "-p"])
        .arg(&f.path)
        .args(["get", "/missing"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "Error: No value at JSON pointer: /missing"
    );
}
