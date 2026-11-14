//! Symbol-level AST deltas for Ralph iterations (Ф3.2, Ф3.3).
//!
//! Mirrors ob2h's `ast_diff` / `record_ast_delta` semantics on the fork side: a
//! snapshot of declared symbols (keyed `file|label|type`) is diffed across an
//! iteration, and the changed symbol labels feed the findings staleness pass.
//! This replaces raw git-diff inspection for validation with a *symbolic*
//! (signature-aware) diff, so a rename or a changed signature is visible even
//! when the textual diff is large.

use omnesagent_kag::{AstCodeExtractor, AstScanResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Symbol node kinds tracked for signature-level diffs.
const TRACKED_TYPES: &[&str] = &[
    "Function",
    "Method",
    "Struct",
    "Class",
    "Interface",
    "Trait",
    "Table",
];

/// Source extensions walked when a `paths_scope` entry is a directory.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "php", "sql", "rb", "java", "kt", "swift", "c",
    "h", "cc", "cpp", "hpp", "cs",
];

/// A single declared symbol observed at a point in time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSig {
    pub label: String,
    pub node_type: String,
    pub file: String,
    /// The declaration's signature line (or its description as a fallback).
    pub signature: String,
}

/// Snapshot of all tracked symbols, keyed `file|label|type`.
pub type SymbolSnapshot = BTreeMap<String, SymbolSig>;

/// How a symbol changed between two snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
}

impl ChangeKind {
    /// Stable lowercase token (matches the serialized form and ob2h output).
    pub fn as_str(self) -> &'static str {
        match self {
            ChangeKind::Added => "added",
            ChangeKind::Modified => "modified",
            ChangeKind::Removed => "removed",
        }
    }
}

/// A single symbol-level change between two snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolChange {
    pub key: String,
    pub label: String,
    pub node_type: String,
    pub file: String,
    pub kind: ChangeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig_after: Option<String>,
}

/// Build a symbol snapshot from `(rel_path, content)` pairs.
pub fn snapshot_sources<'a, I>(files: I) -> SymbolSnapshot
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let extractor = AstCodeExtractor::new();
    let mut snapshot = SymbolSnapshot::new();

    for (rel_path, content) in files {
        let rel_path = normalize_path(rel_path);
        let mut scan = AstScanResult::default();
        extractor.parse_file(&rel_path, content, &mut scan);
        let lines: Vec<&str> = content.lines().collect();

        for node in &scan.nodes {
            if !TRACKED_TYPES.contains(&node.node_type.as_str()) {
                continue;
            }
            let line = node
                .line_start
                .checked_sub(1)
                .and_then(|idx| lines.get(idx))
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty());
            let signature = line.unwrap_or_else(|| node.description.clone());
            let key = format!("{}|{}|{}", normalize_path(&node.file_path), node.label, node.node_type);
            snapshot.insert(
                key,
                SymbolSig {
                    label: node.label.clone(),
                    node_type: node.node_type.clone(),
                    file: normalize_path(&node.file_path),
                    signature,
                },
            );
        }
    }

    snapshot
}

/// Build a snapshot for the given `paths_scope` entries (files or directories).
///
/// Missing paths and unreadable files are skipped — a broken scope entry must
/// never abort the Ralph cycle.
pub fn snapshot_paths(root: &Path, rel_paths: &[String]) -> SymbolSnapshot {
    let mut owned: Vec<(String, String)> = Vec::new();
    for rel in rel_paths {
        // Paths are stored relative to the repo root in openspec/tasks.md.
        let candidate = if Path::new(rel).is_absolute() {
            Path::new(rel).to_path_buf()
        } else {
            root.join(rel)
        };
        if candidate.is_file() {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                owned.push((normalize_path(rel), content));
            }
        } else if candidate.is_dir() {
            collect_source_files(root, &candidate, &mut owned);
        }
    }

    let refs: Vec<(&str, &str)> = owned
        .iter()
        .map(|(path, content)| (path.as_str(), content.as_str()))
        .collect();
    snapshot_sources(refs)
}

/// Recursively collect source files under `dir`, as root-relative `(path, content)`.
fn collect_source_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            collect_source_files(root, &path, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SOURCE_EXTENSIONS.contains(&ext) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let rel = path.strip_prefix(root).unwrap_or(&path);
                    out.push((normalize_path(&rel.to_string_lossy()), content));
                }
            }
        }
    }
}

/// Diff two snapshots, returning added / modified / removed symbols.
///
/// Output is sorted by key for deterministic journal records.
pub fn diff_snapshots(before: &SymbolSnapshot, after: &SymbolSnapshot) -> Vec<SymbolChange> {
    let mut changes = Vec::new();

    for (key, new_sig) in after {
        match before.get(key) {
            None => changes.push(SymbolChange {
                key: key.clone(),
                label: new_sig.label.clone(),
                node_type: new_sig.node_type.clone(),
                file: new_sig.file.clone(),
                kind: ChangeKind::Added,
                sig_before: None,
                sig_after: Some(new_sig.signature.clone()),
            }),
            Some(old_sig) if old_sig.signature != new_sig.signature => changes.push(SymbolChange {
                key: key.clone(),
                label: new_sig.label.clone(),
                node_type: new_sig.node_type.clone(),
                file: new_sig.file.clone(),
                kind: ChangeKind::Modified,
                sig_before: Some(old_sig.signature.clone()),
                sig_after: Some(new_sig.signature.clone()),
            }),
            Some(_) => {}
        }
    }

    for (key, old_sig) in before {
        if !after.contains_key(key) {
            changes.push(SymbolChange {
                key: key.clone(),
                label: old_sig.label.clone(),
                node_type: old_sig.node_type.clone(),
                file: old_sig.file.clone(),
                kind: ChangeKind::Removed,
                sig_before: Some(old_sig.signature.clone()),
                sig_after: None,
            });
        }
    }

    changes.sort_by(|a, b| a.key.cmp(&b.key));
    changes
}

/// Labels of every symbol touched by `changes` (any kind).
///
/// Used when attaching symbols to a freshly recorded finding.
pub fn all_labels(changes: &[SymbolChange]) -> BTreeSet<String> {
    changes.iter().map(|c| c.label.clone()).collect()
}

/// Labels of symbols that were modified or removed (added symbols are ignored).
///
/// Used by the staleness pass: only *existing* symbols being changed or dropped
/// can invalidate a prior finding that referenced them (ob2h FR-K5 semantics).
pub fn changed_labels(changes: &[SymbolChange]) -> BTreeSet<String> {
    changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::Modified | ChangeKind::Removed))
        .map(|c| c.label.clone())
        .collect()
}

/// Render a human-readable symbol-level diff (mirrors ob2h `ast_diff` output).
pub fn render_ast_diff(changes: &[SymbolChange]) -> String {
    if changes.is_empty() {
        return "no symbol changes".to_string();
    }
    let mut out = format!("ast_diff: {} change(s)", changes.len());
    for change in changes {
        out.push_str(&format!(
            "\n- [{}] {}:{} ({})",
            change.kind.as_str(),
            change.node_type,
            change.label,
            change.file
        ));
    }
    out
}

/// Normalize a path to forward slashes for stable snapshot keys.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(src: &str) -> SymbolSnapshot {
        snapshot_sources(vec![("src/lib.rs", src)])
    }

    #[test]
    fn added_symbol_is_detected() {
        let before = snapshot("pub fn alpha() {}\n");
        let after = snapshot("pub fn alpha() {}\npub fn beta() {}\n");
        let changes = diff_snapshots(&before, &after);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].label, "beta");
        assert_eq!(changes[0].kind, ChangeKind::Added);
        assert!(changes[0].sig_before.is_none());
    }

    #[test]
    fn changed_signature_is_modified() {
        let before = snapshot("pub fn alpha(x: i32) -> i32 { x }\n");
        let after = snapshot("pub fn alpha(x: i64) -> i64 { x }\n");
        let changes = diff_snapshots(&before, &after);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, ChangeKind::Modified);
        assert_ne!(changes[0].sig_before, changes[0].sig_after);
        assert_eq!(changed_labels(&changes), BTreeSet::from(["alpha".to_string()]));
    }

    #[test]
    fn removed_symbol_is_detected_and_marks_stale_label() {
        let before = snapshot("pub fn alpha() {}\npub fn beta() {}\n");
        let after = snapshot("pub fn alpha() {}\n");
        let changes = diff_snapshots(&before, &after);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, ChangeKind::Removed);
        assert!(changed_labels(&changes).contains("beta"));
    }

    #[test]
    fn unchanged_symbols_produce_no_changes() {
        let before = snapshot("pub fn alpha() {}\n");
        let after = snapshot("pub fn alpha() {}\n");
        assert!(diff_snapshots(&before, &after).is_empty());
    }

    #[test]
    fn added_symbols_are_not_staleness_labels() {
        let before = snapshot("");
        let after = snapshot("pub fn alpha() {}\n");
        let changes = diff_snapshots(&before, &after);
        assert!(changed_labels(&changes).is_empty());
        assert_eq!(all_labels(&changes), BTreeSet::from(["alpha".to_string()]));
    }

    #[test]
    fn render_is_stable_and_readable() {
        let before = snapshot("pub fn alpha() {}\n");
        let after = snapshot("pub fn beta() {}\n");
        let rendered = render_ast_diff(&diff_snapshots(&before, &after));
        assert!(rendered.contains("[added] Function:beta"));
        assert!(rendered.contains("[removed] Function:alpha"));
    }

    #[test]
    fn snapshot_paths_reads_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "pub fn alpha() {}\n").unwrap();

        let by_dir = snapshot_paths(dir.path(), &["src".to_string()]);
        assert!(by_dir.keys().any(|k| k.starts_with("src/lib.rs|alpha|Function")));

        let by_file = snapshot_paths(dir.path(), &["src/lib.rs".to_string()]);
        assert_eq!(by_dir, by_file);
    }
}
