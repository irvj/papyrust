//! Discovery of a book project's on-disk layout.
//!
//! Given a project root, [`ProjectLayout::discover`] returns the paths of
//! `book.toml`, `cover.jpg` (if present), and ordered Markdown files for
//! each section. It does not read or parse those files — that is the
//! responsibility of [`crate::config`] and [`crate::parse`].
//!
//! Missing optional sections (`front-matter/`, `back-matter/`, missing
//! cover) produce empty results rather than errors. Whether those
//! omissions are acceptable for a given operation is a validation
//! concern (see [`crate::validate`]).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    /// Expected path of `book.toml`. May or may not exist on disk —
    /// existence is checked when the config is loaded.
    pub book_toml: PathBuf,
    pub cover: Option<PathBuf>,
    pub front_matter: Vec<PathBuf>,
    pub chapters: Vec<PathBuf>,
    pub back_matter: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project root not found or not a directory: {}", .0.display())]
    RootNotADirectory(PathBuf),

    #[error("could not read {}: {source}", dir.display())]
    ReadDir {
        dir: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl ProjectLayout {
    pub fn discover(root: &Path) -> Result<Self, ProjectError> {
        if !root.is_dir() {
            return Err(ProjectError::RootNotADirectory(root.to_path_buf()));
        }

        let book_toml = root.join("book.toml");
        let cover_candidate = root.join("cover.jpg");
        let cover = cover_candidate.is_file().then_some(cover_candidate);

        let front_matter = list_markdown(&root.join("front-matter"))?;
        let chapters = list_markdown(&root.join("chapters"))?;
        let back_matter = list_markdown(&root.join("back-matter"))?;

        Ok(Self {
            root: root.to_path_buf(),
            book_toml,
            cover,
            front_matter,
            chapters,
            back_matter,
        })
    }
}

/// List `.md` files in `dir`, ordered by leading numeric prefix.
/// Returns an empty Vec if `dir` does not exist or is not a directory.
fn list_markdown(dir: &Path) -> Result<Vec<PathBuf>, ProjectError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let read = std::fs::read_dir(dir).map_err(|source| ProjectError::ReadDir {
        dir: dir.to_path_buf(),
        source,
    })?;
    let mut entries: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|p| sort_key(p));
    Ok(entries)
}

/// Sort key: leading numeric prefix first (unprefixed files sort last),
/// then filename for stable ordering.
fn sort_key(path: &Path) -> (u32, String) {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let prefix = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| {
            let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<u32>().ok()
        })
        .unwrap_or(u32::MAX);
    (prefix, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    fn touch(dir: &Path, name: &str) {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, "").unwrap();
    }

    #[test]
    fn discovers_full_project() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "book.toml");
        touch(root, "cover.jpg");
        touch(root, "front-matter/01-dedication.md");
        touch(root, "front-matter/02-epigraph.md");
        touch(root, "chapters/01-one.md");
        touch(root, "chapters/02-two.md");
        touch(root, "back-matter/01-about.md");

        let layout = ProjectLayout::discover(root).unwrap();
        assert_eq!(layout.book_toml, root.join("book.toml"));
        assert_eq!(layout.cover, Some(root.join("cover.jpg")));
        assert_eq!(layout.front_matter.len(), 2);
        assert_eq!(layout.chapters.len(), 2);
        assert_eq!(layout.back_matter.len(), 1);
    }

    #[test]
    fn discovers_minimal_project() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "book.toml");
        touch(root, "chapters/01-one.md");

        let layout = ProjectLayout::discover(root).unwrap();
        assert!(layout.cover.is_none());
        assert!(layout.front_matter.is_empty());
        assert_eq!(layout.chapters.len(), 1);
        assert!(layout.back_matter.is_empty());
    }

    #[test]
    fn ignores_non_markdown_files() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "chapters/01-one.md");
        touch(root, "chapters/notes.txt");
        touch(root, "chapters/.DS_Store");

        let layout = ProjectLayout::discover(root).unwrap();
        assert_eq!(layout.chapters.len(), 1);
        assert_eq!(
            layout.chapters[0].file_name().unwrap().to_string_lossy(),
            "01-one.md"
        );
    }

    #[test]
    fn orders_by_numeric_prefix_not_lexicographic() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "chapters/02-b.md");
        touch(root, "chapters/10-j.md");
        touch(root, "chapters/01-a.md");

        let layout = ProjectLayout::discover(root).unwrap();
        let names: Vec<_> = layout
            .chapters
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["01-a.md", "02-b.md", "10-j.md"]);
    }

    #[test]
    fn unprefixed_files_sort_last() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "chapters/01-a.md");
        touch(root, "chapters/intro.md");
        touch(root, "chapters/02-b.md");

        let layout = ProjectLayout::discover(root).unwrap();
        let names: Vec<_> = layout
            .chapters
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["01-a.md", "02-b.md", "intro.md"]);
    }

    #[test]
    fn errors_when_root_missing() {
        let tmp = TempDir::new().unwrap();
        let bogus = tmp.path().join("does-not-exist");
        let result = ProjectLayout::discover(&bogus);
        assert!(matches!(result, Err(ProjectError::RootNotADirectory(_))));
    }

    #[test]
    fn ignores_subdirectories_in_content_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        touch(root, "chapters/01-a.md");
        fs::create_dir_all(root.join("chapters/scratch")).unwrap();
        touch(root, "chapters/scratch/wip.md");

        let layout = ProjectLayout::discover(root).unwrap();
        assert_eq!(layout.chapters.len(), 1);
    }
}
