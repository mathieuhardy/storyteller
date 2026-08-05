//! Shared test helpers.

use std::path::{Path, PathBuf};

use storyteller_core::index::Index;
use storyteller_core::project::Project;

/// A copy of the reference fixture project, in a temporary folder.
///
/// Tests always work on a copy: opening an index writes `cache.sqlite` next to
/// the entries, and a fixture in the repo must stay pristine.
pub struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path().join("la-felure");
        copy_dir(&fixture_source(), &root);
        Self { _dir: dir, root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn project(&self) -> Project {
        Project::open(&self.root).expect("open project")
    }

    /// An index built from the fixture, ready to query.
    pub fn indexed(&self) -> (Project, Index) {
        let project = self.project();
        let mut index = Index::open(project.root()).expect("open index");
        index
            .rebuild_from_project(&project)
            .expect("rebuild the index");
        (project, index)
    }

    pub fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    pub fn remove(&self, relative: &str) {
        std::fs::remove_file(self.root.join(relative)).unwrap();
    }
}

/// The reference project, shared with `storyteller-server`'s tests.
pub fn fixture_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/sample-project")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
