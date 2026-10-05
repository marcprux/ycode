// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use ycode_project::{Error, ProjectDocument};
// Synthetic filesystem fixtures.
const SOURCE: &str =
    "{files:[],'default-configuration':'Debug',localizations:{development:'en'}}\n";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ycode-project-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("project.xcproj"), SOURCE).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn saves_are_optimistic_atomic_and_noops_do_not_rewrite() {
    let f = Fixture::new();
    let path = f.0.join("project.xcproj");
    let mut doc = ProjectDocument::open(&path).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    doc.save().unwrap();
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    doc.set_build_setting(None, "A", "b".into()).unwrap();
    doc.save().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), doc.source());
    doc.set_build_setting(None, "A", "c".into()).unwrap();
    fs::write(&path, SOURCE.to_owned() + "// external\n").unwrap();
    assert!(matches!(doc.save(), Err(Error::ConcurrentChange)));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        SOURCE.to_owned() + "// external\n"
    );
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
}
#[test]
fn new_output_refuses_to_clobber_and_cleans_temporary_file() {
    let f = Fixture::new();
    let path = f.0.join("output.xcproj");
    let doc = ProjectDocument::parse(SOURCE).unwrap();
    doc.write_new(&path).unwrap();
    assert!(doc.write_new(&path).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), SOURCE);
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 2);
}
#[cfg(unix)]
#[test]
fn save_retains_permissions_and_symlink_identity() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    let path = f.0.join("project.xcproj");
    let link = f.0.join("link.xcproj");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    symlink(&path, &link).unwrap();
    let mut doc = ProjectDocument::open(&link).unwrap();
    doc.set_build_setting(None, "A", "b".into()).unwrap();
    doc.save().unwrap();
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}
