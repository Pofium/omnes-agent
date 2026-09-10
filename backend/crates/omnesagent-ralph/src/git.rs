//! Git integration for Ralph: atomic commits, status, diff, and scoping checks.
//!
//! Conforms to §7.5, FR-A13, NFR-A1 of specification.

use omnesagent_config::ralph::RalphCommitFormat;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    Execution(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Scope violation: modified file '{0}' is outside paths_scope {1:?}")]
    ScopeViolation(String, Vec<String>),
}

/// Helper for git operations within the target repository.
pub struct GitRepo<'a> {
    pub root: &'a Path,
}

impl<'a> GitRepo<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }

    fn run_git(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(self.root)
            .stdin(std::process::Stdio::null())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::Execution(format!(
                "git {} failed with exit code {:?}: {}",
                args.join(" "),
                output.status.code(),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Retrieve the current HEAD commit hash.
    pub fn rev_parse_head(&self) -> Result<String, GitError> {
        self.run_git(&["rev-parse", "HEAD"])
    }

    /// Get git diff of working directory against HEAD or staged.
    pub fn diff(&self, path_filter: Option<&str>) -> Result<String, GitError> {
        let mut args = vec!["diff", "HEAD"];
        if let Some(pf) = path_filter {
            args.push("--");
            args.push(pf);
        }
        self.run_git(&args)
    }

    /// List files modified or untracked since HEAD.
    pub fn list_changed_files(&self) -> Result<Vec<String>, GitError> {
        let stdout = self.run_git(&["status", "--porcelain"])?;
        let mut changed = Vec::new();
        for raw_line in stdout.lines() {
            let line = raw_line.trim_end();
            let trimmed_start = line.trim_start();
            if let Some(space_idx) = trimmed_start.find(' ') {
                let file = trimmed_start[space_idx..].trim();
                // Handle rename syntax "old -> new"
                let target_file = if let Some(pos) = file.find("->") {
                    file[pos + 2..].trim()
                } else {
                    file
                };
                let clean = target_file.trim_matches('"').to_string();
                if !clean.is_empty() {
                    changed.push(clean);
                }
            }
        }
        Ok(changed)
    }

    /// Validate that all modified files comply with `paths_scope` whitelist (§5.4).
    pub fn check_paths_scope(&self, paths_scope: &[String]) -> Result<(), GitError> {
        if paths_scope.is_empty() {
            return Ok(());
        }

        let changed = self.list_changed_files()?;
        for file in changed {
            let normalized = file.trim_matches('"').replace('\\', "/");
            // openspec metadata directory is always permitted
            if normalized.starts_with("openspec/") || normalized.starts_with(".git") {
                continue;
            }

            let in_scope = paths_scope.iter().any(|scope| {
                let scope_norm = scope.trim_matches('"').replace('\\', "/");
                normalized == scope_norm || normalized.starts_with(&scope_norm)
            });

            if !in_scope {
                return Err(GitError::ScopeViolation(file, paths_scope.to_vec()));
            }
        }
        Ok(())
    }

    /// Commit the iteration changes with the configured message format (§7.5).
    pub fn commit_iteration(
        &self,
        task_id: &str,
        iteration_n: usize,
        summary: &str,
        format: RalphCommitFormat,
    ) -> Result<String, GitError> {
        // Stage all modifications except untracked outside scope
        self.run_git(&["add", "-A"])?;

        let message = match format {
            RalphCommitFormat::Cc => {
                format!("feat(ralph/{}): i{} — {}", task_id, iteration_n, summary)
            }
            RalphCommitFormat::Legacy => {
                format!("ralph({}, i{}): {}", task_id, iteration_n, summary)
            }
        };

        self.run_git(&["commit", "-m", &message])?;
        self.rev_parse_head()
    }

    /// Discard all uncommitted working directory changes.
    pub fn revert_working_copy(&self) -> Result<(), GitError> {
        self.run_git(&["checkout", "."])?;
        self.run_git(&["clean", "-fd"])?;
        Ok(())
    }
}
