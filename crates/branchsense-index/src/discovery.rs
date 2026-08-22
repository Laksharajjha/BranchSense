//! Deterministic, symlink-safe Java source discovery.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{DiscoveryError, Result};

/// A discovered Java source file represented relative to the repository root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredFile {
    relative_path: PathBuf,
    absolute_path: PathBuf,
}

impl DiscoveredFile {
    fn new(root: &Path, absolute_path: PathBuf) -> Result<Self> {
        let relative_path = absolute_path
            .strip_prefix(root)
            .map_err(|_| DiscoveryError::OutsideRoot {
                path: absolute_path.clone(),
                root: root.to_path_buf(),
            })?
            .to_path_buf();
        Ok(Self { relative_path, absolute_path })
    }

    /// Returns the repository-relative source path.
    #[must_use]
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }
    /// Returns the canonical filesystem path.
    #[must_use]
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }
}

/// Configures repository source discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryOptions {
    ignored_directories: BTreeSet<String>,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            ignored_directories: ["target", "build", ".gradle", ".idea", ".git"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }
}

impl DiscoveryOptions {
    /// Creates default discovery options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds a directory basename to the ignored set.
    #[must_use]
    pub fn ignore_directory(mut self, name: impl Into<String>) -> Self {
        self.ignored_directories.insert(name.into());
        self
    }
    /// Removes a directory basename from the ignored set.
    #[must_use]
    pub fn include_directory(mut self, name: &str) -> Self {
        self.ignored_directories.remove(name);
        self
    }
    /// Returns ignored directory basenames in deterministic order.
    #[must_use]
    pub fn ignored_directories(&self) -> &BTreeSet<String> {
        &self.ignored_directories
    }
}

/// Output of one discovery pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryResult {
    root: PathBuf,
    files: Vec<DiscoveredFile>,
    skipped: usize,
}

impl DiscoveryResult {
    /// Returns the canonical project root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Returns Java source files in relative-path order.
    #[must_use]
    pub fn files(&self) -> &[DiscoveredFile] {
        &self.files
    }
    /// Returns skipped filesystem entries.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.skipped
    }
}

/// Discovers Java sources under a repository root.
#[derive(Clone, Debug)]
pub struct SourceDiscovery {
    options: DiscoveryOptions,
}

impl SourceDiscovery {
    /// Creates a discovery service with explicit options.
    #[must_use]
    pub const fn new(options: DiscoveryOptions) -> Self {
        Self { options }
    }
    /// Returns the configured discovery options.
    #[must_use]
    pub const fn options(&self) -> &DiscoveryOptions {
        &self.options
    }

    /// Discovers `.java` files below `path`.
    ///
    /// A file path is treated as a one-file project. Directory symlinks and
    /// symlinked files are skipped rather than followed, preventing cycles and
    /// identity ambiguity. Results are sorted by repository-relative path.
    ///
    /// # Errors
    /// Returns [`DiscoveryError`] when the root cannot be canonicalized or
    /// traversed.
    pub fn discover(&self, path: impl AsRef<Path>) -> Result<DiscoveryResult> {
        let requested = path.as_ref();
        let metadata = fs::symlink_metadata(requested)
            .map_err(|source| DiscoveryError::Io { path: requested.to_path_buf(), source })?;
        if metadata.file_type().is_symlink() {
            return Err(DiscoveryError::InvalidRoot(requested.to_path_buf()));
        }
        let root = fs::canonicalize(requested)
            .map_err(|source| DiscoveryError::Io { path: requested.to_path_buf(), source })?;
        if root.is_file() {
            if root.extension().is_some_and(|extension| extension == "java") {
                let project_root = root.parent().unwrap_or(&root).to_path_buf();
                return Ok(DiscoveryResult {
                    files: vec![DiscoveredFile::new(&project_root, root.clone())?],
                    root: project_root,
                    skipped: 0,
                });
            }
            return Ok(DiscoveryResult { root, files: Vec::new(), skipped: 1 });
        }
        if !root.is_dir() {
            return Err(DiscoveryError::InvalidRoot(root));
        }
        let mut files = Vec::new();
        let mut skipped = 0;
        self.visit(&root, &root, &mut files, &mut skipped)?;
        files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(DiscoveryResult { root, files, skipped })
    }

    fn visit(
        &self,
        root: &Path,
        directory: &Path,
        files: &mut Vec<DiscoveredFile>,
        skipped: &mut usize,
    ) -> Result<()> {
        let mut entries = fs::read_dir(directory)
            .map_err(|source| DiscoveryError::Io { path: directory.to_path_buf(), source })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|source| DiscoveryError::Io { path: directory.to_path_buf(), source })?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|source| DiscoveryError::Io { path: path.clone(), source })?;
            if metadata.file_type().is_symlink() {
                *skipped += 1;
                continue;
            }
            if metadata.is_dir() {
                if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| self.options.ignored_directories.contains(name))
                {
                    *skipped += 1;
                    continue;
                }
                self.visit(root, &path, files, skipped)?;
            } else if metadata.is_file()
                && path.extension().is_some_and(|extension| extension == "java")
            {
                files.push(DiscoveredFile::new(
                    root,
                    fs::canonicalize(&path)
                        .map_err(|source| DiscoveryError::Io { path: path.clone(), source })?,
                )?);
            }
        }
        Ok(())
    }
}
