//! Co-Change (Temporal Coupling) discovery from Git commit history.
//!
//! Analyzes recent git commits to find pairs of files frequently changed together.
//! Creates bidirectional `CO_CHANGES_WITH` edges in the knowledge graph
//! with `provenance = 'GIT_HISTORY'` and confidence proportional to commit frequency.

use rusqlite::{params, Connection, Result};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;

/// Default commit history limit to scan.
pub const DEFAULT_COMMIT_LIMIT: usize = 100;
/// Minimum co-commits between two files to establish a co-change relation.
pub const DEFAULT_MIN_CO_COMMITS: usize = 3;
/// Maximum files in a single commit (commits larger than this are considered bulk/refactor and skipped).
pub const DEFAULT_MAX_FILES_PER_COMMIT: usize = 10;

/// Parses `git log --name-only --pretty=format:COMMIT:%H` output and extracts co-changing file pairs.
///
/// Returns list of tuples `(file_a, file_b, co_commit_count, confidence)`.
pub fn extract_cochange_pairs_from_log(
    git_log_text: &str,
    min_co_commits: usize,
    max_files_per_commit: usize,
) -> Vec<(String, String, usize, f64)> {
    let mut current_commit_files: BTreeSet<String> = BTreeSet::new();
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

    let flush_commit = |files: &mut BTreeSet<String>, pairs: &mut HashMap<(String, String), usize>| {
        if files.len() >= 2 && files.len() <= max_files_per_commit {
            let list: Vec<&String> = files.iter().collect();
            for i in 0..list.len() {
                for j in (i + 1)..list.len() {
                    let a = (*list[i]).clone();
                    let b = (*list[j]).clone();
                    let key = if a <= b { (a, b) } else { (b, a) };
                    *pairs.entry(key).or_insert(0) += 1;
                }
            }
        }
        files.clear();
    };

    for raw_line in git_log_text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("COMMIT:") {
            flush_commit(&mut current_commit_files, &mut pair_counts);
        } else {
            let normalized = line.replace('\\', "/");
            current_commit_files.insert(normalized);
        }
    }
    flush_commit(&mut current_commit_files, &mut pair_counts);

    let mut result: Vec<(String, String, usize, f64)> = pair_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_co_commits)
        .map(|((a, b), count)| {
            let confidence = (count as f64 / 10.0).min(1.0);
            (a, b, count, confidence)
        })
        .collect();

    // Sort descending by co-commit count
    result.sort_by(|a, b| b.2.cmp(&a.2));
    result
}

/// Executes `git log` on a git repository directory and extracts co-changing file pairs.
pub fn extract_cochanges_from_git_repo(
    repo_path: &Path,
    commit_limit: usize,
    min_co_commits: usize,
    max_files_per_commit: usize,
) -> std::io::Result<Vec<(String, String, usize, f64)>> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-n{}", commit_limit),
            "--name-only",
            "--pretty=format:COMMIT:%H",
        ])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let log_text = String::from_utf8_lossy(&out.stdout);
            Ok(extract_cochange_pairs_from_log(
                &log_text,
                min_co_commits,
                max_files_per_commit,
            ))
        }
        _ => Ok(Vec::new()), // Graceful fallback if git is not present or directory is not a git repo
    }
}

/// Persists `CO_CHANGES_WITH` edges into SQLite `graph_edges` table.
///
/// Matches files to their primary `graph_nodes` entries in the project.
pub fn sync_cochange_edges(
    conn: &Connection,
    project_id: &str,
    pairs: &[(String, String, usize, f64)],
) -> Result<usize> {
    if pairs.is_empty() {
        return Ok(0);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let mut edge_count = 0;

    let mut find_node_stmt = conn.prepare_cached(
        r#"
        SELECT id FROM graph_nodes
        WHERE project_id = ?1
          AND (file_path = ?2 OR file_path LIKE ?3)
          AND (deleted_at IS NULL OR deleted_at = '')
        ORDER BY CASE WHEN file_path = ?2 THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )?;

    let mut insert_edge_stmt = conn.prepare_cached(
        r#"
        INSERT INTO graph_edges (source_id, target_id, label, weight, contexts, project_id, provenance, confidence, created_at, updated_at)
        VALUES (?1, ?2, 'CO_CHANGES_WITH', ?3, ?4, ?5, 'GIT_HISTORY', ?3, ?6, ?6)
        ON CONFLICT(source_id, target_id, label) DO UPDATE SET
            weight = excluded.weight,
            contexts = excluded.contexts,
            provenance = excluded.provenance,
            confidence = excluded.confidence,
            updated_at = excluded.updated_at
        "#,
    )?;

    for (file_a, file_b, count, confidence) in pairs {
        let pattern_a = format!("%{}", file_a);
        let pattern_b = format!("%{}", file_b);

        let pk_a: Option<i64> = find_node_stmt
            .query_row(params![project_id, file_a, pattern_a], |r| r.get(0))
            .ok();
        let pk_b: Option<i64> = find_node_stmt
            .query_row(params![project_id, file_b, pattern_b], |r| r.get(0))
            .ok();

        if let (Some(a), Some(b)) = (pk_a, pk_b) {
            if a != b {
                let context = format!("{{\"co_commits\":{}}}", count);

                // Insert forward edge a -> b
                insert_edge_stmt.execute(params![a, b, confidence, context, project_id, now])?;
                // Insert backward edge b -> a
                insert_edge_stmt.execute(params![b, a, confidence, context, project_id, now])?;
                edge_count += 2;
            }
        }
    }

    Ok(edge_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_pairs_above_threshold() {
        let log = r#"
COMMIT:1111111
src/auth/service.rs
src/auth/token.rs
src/db/user.rs

COMMIT:2222222
src/auth/service.rs
src/auth/token.rs

COMMIT:3333333
src/auth/service.rs
src/auth/token.rs
src/api/login.rs

COMMIT:4444444
src/db/user.rs
src/other.rs
"#;

        let pairs = extract_cochange_pairs_from_log(log, 3, 10);
        assert_eq!(pairs.len(), 1);
        let (a, b, count, conf) = &pairs[0];
        assert_eq!(a, "src/auth/service.rs");
        assert_eq!(b, "src/auth/token.rs");
        assert_eq!(*count, 3);
        assert!((conf - 0.3).abs() < 1e-6);
    }

    #[test]
    fn skips_commits_exceeding_max_files() {
        let log = r#"
COMMIT:bulk_format
f1.rs
f2.rs
f3.rs
f4.rs
f5.rs

COMMIT:normal1
f1.rs
f2.rs

COMMIT:normal2
f1.rs
f2.rs
"#;

        // max_files_per_commit = 4: bulk_format with 5 files is skipped
        let pairs = extract_cochange_pairs_from_log(log, 2, 4);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].2, 2);

        // If min_co_commits = 3, it won't qualify because bulk commit was skipped
        let pairs_3 = extract_cochange_pairs_from_log(log, 3, 4);
        assert!(pairs_3.is_empty());
    }

    #[test]
    fn syncs_edges_to_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::migrate(&conn).unwrap();

        conn.execute(
            "INSERT INTO projects (id, name, root_path, created_at, updated_at) VALUES ('p1', 'demo', 'c:/demo', 't', 't')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO graph_nodes (id, node_id, label, node_type, file_path, project_id, provenance, confidence, created_at, updated_at) \
             VALUES (1, 'n1', 'service.rs', 'File', 'src/auth/service.rs', 'p1', 'ast', 1.0, 't', 't')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO graph_nodes (id, node_id, label, node_type, file_path, project_id, provenance, confidence, created_at, updated_at) \
             VALUES (2, 'n2', 'token.rs', 'File', 'src/auth/token.rs', 'p1', 'ast', 1.0, 't', 't')",
            [],
        ).unwrap();

        let pairs = vec![(
            "src/auth/service.rs".to_string(),
            "src/auth/token.rs".to_string(),
            5,
            0.5,
        )];

        let synced = sync_cochange_edges(&conn, "p1", &pairs).unwrap();
        assert_eq!(synced, 2, "inserted forward and backward edges");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE label = 'CO_CHANGES_WITH' AND project_id = 'p1'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 2);
    }
}
