//! OpenSpec delta-spec workspace layout and utilities.
//!
//! Conforms to §1.2, §4, §5.4 of specification.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenSpecError {
    #[error("Change directory '{0}' not found")]
    NotFound(PathBuf),
    #[error("Missing tasks.md in '{0}'")]
    MissingTasks(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Helper for accessing files within an `openspec/changes/<slug>/` workspace.
#[derive(Debug, Clone)]
pub struct OpenSpecWorkspace {
    pub root: PathBuf,
    pub slug: String,
}

impl OpenSpecWorkspace {
    pub fn new(repo_root: &Path, slug: &str) -> Self {
        let root = repo_root.join("openspec").join("changes").join(slug);
        Self {
            root,
            slug: slug.to_string(),
        }
    }

    pub fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }

    pub fn journal_path(&self) -> PathBuf {
        self.root.join("journal.jsonl")
    }

    pub fn tasks_path(&self) -> PathBuf {
        self.root.join("tasks.md")
    }

    pub fn specs_dir(&self) -> PathBuf {
        self.root.join("specs")
    }

    /// Ensure the change directory exists.
    pub fn init(&self) -> Result<(), OpenSpecError> {
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }

    /// Read raw content of `tasks.md`.
    pub fn read_tasks_markdown(&self) -> Result<String, OpenSpecError> {
        let path = self.tasks_path();
        if !path.exists() {
            return Err(OpenSpecError::MissingTasks(path));
        }
        Ok(std::fs::read_to_string(path)?)
    }
}
