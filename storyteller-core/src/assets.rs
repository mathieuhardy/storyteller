//! Asset listing and storage under `assets/` (`docs/api.md` §3 "Assets",
//! `docs/glossary.md#asset`). Assets are plain files: no frontmatter, no
//! parsing, no index entry — this module only lists, serves, and writes bytes.

use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::project::{Project, ASSETS_DIR};

/// Image extensions recognized for [`AssetKind::Image`] — the set actually used
/// by the type catalog's `image`/`image-list` fields (`docs/data-model.md`).
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Image,
    File,
}

/// One file under `assets/` — shape of `GET /assets` items.
#[derive(Debug, Clone, Serialize)]
pub struct AssetInfo {
    /// Project-relative path, e.g. `assets/images/aria.jpg`.
    pub path: String,
    pub kind: AssetKind,
    pub size: u64,
}

fn kind_of(path: &Path) -> AssetKind {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if IMAGE_EXTENSIONS.iter().any(|i| i.eq_ignore_ascii_case(ext)) => {
            AssetKind::Image
        }
        _ => AssetKind::File,
    }
}

/// Keeps only the filename, so an uploaded name can never escape `assets/` or
/// place itself in a subfolder the caller didn't ask to browse into.
fn sanitize_filename(name: &str) -> Option<String> {
    let base = Path::new(name).file_name()?.to_str()?.to_string();
    (!base.is_empty()).then_some(base)
}

impl Project {
    /// Lists every file under `assets/`, sorted by path. Empty (not an error)
    /// when the folder does not exist yet — a fresh project has no assets.
    pub fn list_assets(&self) -> Vec<AssetInfo> {
        let root = self.root().join(ASSETS_DIR);
        if !root.is_dir() {
            return Vec::new();
        }
        let mut assets: Vec<AssetInfo> = WalkDir::new(&root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let relative = relative_to_root(self.root(), entry.path())?;
                let size = entry.metadata().ok()?.len();
                Some(AssetInfo {
                    kind: kind_of(entry.path()),
                    path: relative,
                    size,
                })
            })
            .collect();
        assets.sort_by(|a, b| a.path.cmp(&b.path));
        assets
    }

    /// Absolute path of an asset, refusing anything outside `assets/` — a
    /// stricter check than [`Project::absolute_path`], which only guards the
    /// project root.
    pub fn asset_absolute_path(&self, relative: &str) -> Result<PathBuf> {
        let absolute = self.absolute_path(relative)?;
        let assets_root = self.root().join(ASSETS_DIR);
        if !absolute.starts_with(&assets_root) {
            return Err(Error::PathEscapesProject(PathBuf::from(relative)));
        }
        if !absolute.is_file() {
            return Err(Error::AssetNotFound(relative.to_string()));
        }
        Ok(absolute)
    }

    /// Saves an uploaded asset directly under `assets/` (its name flattened to
    /// a bare filename — see [`sanitize_filename`]). Refuses to overwrite an
    /// existing file, the same identity discipline as entry creation.
    pub fn save_asset(&self, filename: &str, bytes: &[u8]) -> Result<String> {
        let name = sanitize_filename(filename)
            .ok_or_else(|| Error::InvalidTitle(filename.to_string()))?;
        let relative = format!("{ASSETS_DIR}/{name}");
        let absolute = self.absolute_path(&relative)?;
        if absolute.exists() {
            return Err(Error::AssetExists(relative));
        }
        if let Some(parent) = absolute.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        std::fs::write(&absolute, bytes).map_err(|e| Error::io(&absolute, e))?;
        Ok(relative)
    }
}

/// Project-relative path with `/` separators, or `None` if outside the root.
fn relative_to_root(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with_assets() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("assets/images")).unwrap();
        std::fs::create_dir_all(root.join("assets/maps")).unwrap();
        std::fs::write(root.join("assets/images/aria.jpg"), b"fake-jpeg-bytes").unwrap();
        std::fs::write(root.join("assets/maps/city.png"), b"fake-png-bytes").unwrap();
        std::fs::write(root.join("assets/notes.txt"), b"not an image").unwrap();
        dir
    }

    #[test]
    fn lists_every_file_under_assets_sorted_by_path() {
        let dir = project_with_assets();
        let project = Project::open(dir.path()).unwrap();
        let assets = project.list_assets();
        assert_eq!(
            assets.iter().map(|a| a.path.as_str()).collect::<Vec<_>>(),
            ["assets/images/aria.jpg", "assets/maps/city.png", "assets/notes.txt"]
        );
        assert_eq!(assets[0].kind, AssetKind::Image);
        assert_eq!(assets[2].kind, AssetKind::File);
        assert_eq!(assets[0].size, "fake-jpeg-bytes".len() as u64);
    }

    #[test]
    fn a_project_without_an_assets_folder_lists_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::open(dir.path()).unwrap();
        assert!(project.list_assets().is_empty());
    }

    #[test]
    fn asset_absolute_path_stays_inside_assets_and_rejects_the_rest() {
        let dir = project_with_assets();
        let project = Project::open(dir.path()).unwrap();
        assert!(project.asset_absolute_path("assets/images/aria.jpg").is_ok());
        assert!(matches!(
            project.asset_absolute_path("assets/missing.png"),
            Err(Error::AssetNotFound(_))
        ));
        // A project-relative path that resolves outside `assets/` (even if it
        // exists) is refused: `/assets/{path}` may only ever serve assets.
        std::fs::write(dir.path().join("project.md"), "---\ntype: project\n---\n").unwrap();
        assert!(matches!(
            project.asset_absolute_path("project.md"),
            Err(Error::PathEscapesProject(_))
        ));
    }

    #[test]
    fn save_asset_writes_under_assets_and_refuses_a_collision() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::open(dir.path()).unwrap();

        let relative = project.save_asset("cover.png", b"bytes").unwrap();
        assert_eq!(relative, "assets/cover.png");
        assert_eq!(
            std::fs::read(dir.path().join("assets/cover.png")).unwrap(),
            b"bytes"
        );

        assert!(matches!(
            project.save_asset("cover.png", b"other"),
            Err(Error::AssetExists(_))
        ));
    }

    #[test]
    fn save_asset_flattens_a_path_like_name_to_its_filename() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::open(dir.path()).unwrap();
        let relative = project.save_asset("../../etc/passwd", b"x").unwrap();
        assert_eq!(relative, "assets/passwd");
        assert!(!dir.path().join("etc/passwd").exists());
    }
}
