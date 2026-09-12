//! Сервис управления проектами и проектной AST-памятью.

pub mod watcher;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::info;

pub use watcher::ProjectWatcher;
use crate::ast::{AstCodeExtractor, AstScanResult};
use crate::models::ProjectRecord;
use crate::vector::similarity::top_k;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub project_id: String,
    pub name: String,
    pub root_path: String,
    pub total_nodes: usize,
    pub ast_nodes: usize,
    pub total_edges: usize,
    pub god_nodes: usize,
    pub last_scanned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredProjectNode {
    pub id: i64,
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub file_path: Option<String>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub description: Option<String>,
    pub score: f64,
    pub provenance: String,
    pub is_god_node: bool,
}

#[derive(Clone)]
pub struct ProjectService {
    conn: Arc<Mutex<Connection>>,
    ast_extractor: Arc<AstCodeExtractor>,
    embedder: Option<Arc<dyn crate::embedding::EmbeddingProvider>>,
}

type RawNodeTuple = (i64, String, String, Option<String>, Option<String>);

impl ProjectService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self::new_with_embedder(conn, None)
    }

    pub fn new_with_embedder(
        conn: Arc<Mutex<Connection>>,
        embedder: Option<Arc<dyn crate::embedding::EmbeddingProvider>>,
    ) -> Self {
        Self {
            conn,
            ast_extractor: Arc::new(AstCodeExtractor::new()),
            embedder,
        }
    }

    pub fn with_embedder(mut self, embedder: Arc<dyn crate::embedding::EmbeddingProvider>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    pub fn conn(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    pub fn analyze_impact(
        &self,
        project_id: &str,
        symbol_or_path: &str,
        max_depth: usize,
    ) -> anyhow::Result<crate::graph::ImpactReport> {
        let conn = self.conn.lock();
        Ok(crate::graph::GraphAnalytics::analyze_impact(&conn, project_id, symbol_or_path, max_depth)?)
    }

    pub fn build_context(&self, project_id: &str, query: Option<&str>) -> anyhow::Result<String> {
        let conn = self.conn.lock();
        Ok(crate::graph::GraphAnalytics::build_project_context(&conn, project_id, query)?)
    }

    pub fn generate_project_report(&self, project_id: &str) -> anyhow::Result<crate::graph::ProjectReport> {
        let conn = self.conn.lock();
        Ok(crate::graph::GraphAnalytics::generate_project_report(&conn, project_id)?)
    }

    /// Векторизация узлов графа проекта без эмбеддингов
    pub async fn embed_unembedded_nodes(&self, project_id: &str) -> anyhow::Result<usize> {
        let embedder = match &self.embedder {
            Some(e) => e.clone(),
            None => return Ok(0),
        };

        let nodes: Vec<RawNodeTuple> = {
            let conn = self.conn.lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, node_type, label, file_path, description
                FROM graph_nodes
                WHERE project_id = ?1
                  AND embedding IS NULL
                  AND (deleted_at IS NULL OR deleted_at = '')
                LIMIT 500
                "#,
            )?;
            let rows = stmt.query_map(params![project_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            })?;
            let mut res = Vec::new();
            for r in rows {
                res.push(r?);
            }
            res
        };

        if nodes.is_empty() {
            return Ok(0);
        }

        let mut total_embedded = 0;
        for chunk in nodes.chunks(32) {
            let texts: Vec<String> = chunk
                .iter()
                .map(|(_, ntype, label, fpath, desc)| {
                    let loc = fpath.as_deref().unwrap_or("-");
                    let d = desc.as_deref().unwrap_or("");
                    if d.is_empty() {
                        format!("{ntype} {label} in {loc}")
                    } else {
                        format!("{ntype} {label} in {loc}: {d}")
                    }
                })
                .collect();

            let vecs = embedder.embed(&texts).await?;
            {
                let mut conn = self.conn.lock();
                let tx = conn.transaction()?;
                for ((pk, _, _, _, _), vec) in chunk.iter().zip(vecs.iter()) {
                    let blob = crate::vector::serialize(vec);
                    tx.execute(
                        "UPDATE graph_nodes SET embedding = ?1 WHERE id = ?2",
                        params![blob, pk],
                    )?;
                }
                tx.commit()?;
            }
            total_embedded += chunk.len();
        }

        if total_embedded > 0 {
            info!("ProjectService: векторизовано {} узлов кода проекта '{}'", total_embedded, project_id);
        }
        Ok(total_embedded)
    }

    /// Регистрирует новый проект или обновляет существующий.
    pub fn register_project(
        &self,
        id: &str,
        name: &str,
        root_path: &str,
        description: Option<&str>,
        tech_stack: Option<&[String]>,
    ) -> anyhow::Result<ProjectRecord> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        let abs_path = std::fs::canonicalize(root_path)
            .unwrap_or_else(|_| PathBuf::from(root_path))
            .to_string_lossy()
            .replace('\\', "/");

        let tech_stack_json = tech_stack.map(|ts| serde_json::to_string(ts).unwrap_or_default());

        conn.execute(
            r#"
            INSERT INTO projects (id, name, root_path, description, tech_stack, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                root_path = excluded.root_path,
                description = COALESCE(excluded.description, projects.description),
                tech_stack = COALESCE(excluded.tech_stack, projects.tech_stack),
                updated_at = excluded.updated_at
            "#,
            params![id, name, abs_path, description, tech_stack_json, now, now],
        )?;

        let project = ProjectRecord {
            id: id.to_string(),
            name: name.to_string(),
            root_path: abs_path,
            description: description.map(|s| s.to_string()),
            tech_stack: tech_stack_json,
            active_branch: None,
            last_scanned_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        info!("Зарегистрирован проект '{}' ({}) по пути: {}", name, id, project.root_path);
        Ok(project)
    }

    /// Получить проект по ID.
    pub fn get_project(&self, id: &str) -> anyhow::Result<Option<ProjectRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, description, tech_stack, active_branch, last_scanned_at, created_at, updated_at FROM projects WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ProjectRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                description: row.get(3)?,
                tech_stack: row.get(4)?,
                active_branch: row.get(5)?,
                last_scanned_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Список всех проектов.
    pub fn list_projects(&self) -> anyhow::Result<Vec<ProjectRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, description, tech_stack, active_branch, last_scanned_at, created_at, updated_at FROM projects ORDER BY updated_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ProjectRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                description: row.get(3)?,
                tech_stack: row.get(4)?,
                active_branch: row.get(5)?,
                last_scanned_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    /// Автоматическое определение проекта по рабочей директории.
    pub fn detect_project_by_path(&self, current_dir: &str) -> anyhow::Result<Option<ProjectRecord>> {
        let normalized = PathBuf::from(current_dir);
        let abs_current = std::fs::canonicalize(&normalized)
            .unwrap_or(normalized)
            .to_string_lossy()
            .replace('\\', "/");

        let projects = self.list_projects()?;
        for p in projects {
            if abs_current.starts_with(&p.root_path) {
                return Ok(Some(p));
            }
        }
        Ok(None)
    }

    /// Мапа известных файлов проекта rel_path -> sha256.
    pub fn get_known_files(&self, project_id: &str) -> anyhow::Result<std::collections::HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT rel_path, sha256 FROM project_files WHERE project_id = ?1"
        )?;
        let rows = stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            let (p, h) = r?;
            map.insert(p, h);
        }
        Ok(map)
    }

    /// Автоматически определяет проект по рабочей директории или регистрирует его (Zero-Config).
    pub fn auto_register_or_detect(&self, start_dir: &Path) -> anyhow::Result<ProjectRecord> {
        let root = find_project_root(start_dir);
        let abs_root = std::fs::canonicalize(&root)
            .unwrap_or(root.clone())
            .to_string_lossy()
            .replace('\\', "/");

        // 1. Проверяем, существует ли проект
        {
            let conn = self.conn.lock();
            let mut stmt = conn.prepare(
                "SELECT id, name, root_path, description, tech_stack, active_branch, last_scanned_at, created_at, updated_at FROM projects WHERE root_path = ?1"
            )?;
            let mut rows = stmt.query(params![abs_root])?;
            if let Some(row) = rows.next()? {
                return Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    description: row.get(3)?,
                    tech_stack: row.get(4)?,
                    active_branch: row.get(5)?,
                    last_scanned_at: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                });
            }
        }

        // 2. Детектируем манифест
        let (name, tech_stack, description) = detect_manifest_metadata(&root);
        let mut slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '-' })
            .collect::<String>();
        while slug.contains("--") {
            slug = slug.replace("--", "-");
        }
        let slug = slug.trim_matches('-').to_string();
        let base_id = if slug.is_empty() { "project".to_string() } else { slug };
        let mut project_id = base_id.clone();
        let mut counter = 1;
        while self.get_project(&project_id)?.is_some() {
            project_id = format!("{}-{}", base_id, counter);
            counter += 1;
        }

        let mut project = self.register_project(
            &project_id,
            &name,
            &abs_root,
            description.as_deref(),
            Some(&tech_stack),
        )?;

        // Активная ветка git
        let head_file = root.join(".git").join("HEAD");
        if head_file.exists()
            && let Ok(head_content) = std::fs::read_to_string(&head_file) {
                let branch = head_content.trim().trim_start_matches("ref: refs/heads/").to_string();
                if !branch.is_empty() {
                    let conn = self.conn.lock();
                    let _ = conn.execute(
                        "UPDATE projects SET active_branch = ?1 WHERE id = ?2",
                        params![branch, project.id],
                    );
                    project.active_branch = Some(branch);
                }
            }

        info!("Zero-Config: зарегистрирован проект '{}' ({}) по пути {}", name, project.id, abs_root);
        Ok(project)
    }

    /// Регистрирует проект автоматически по пути с опциональным переопределением ID.
    pub fn register_project_auto(&self, start_dir: &Path, _id_override: Option<&str>) -> anyhow::Result<ProjectRecord> {
        self.auto_register_or_detect(start_dir)
    }

    /// Сканирует кодовую базу проекта через AST и обновляет граф знаний.
    pub fn scan_project(&self, project_id: &str, custom_path: Option<&str>, incremental: bool) -> anyhow::Result<AstScanResult> {
        let project = self.get_project(project_id)?
            .ok_or_else(|| anyhow::anyhow!("Проект с ID '{}' не найден", project_id))?;

        let scan_path = custom_path.unwrap_or(&project.root_path);
        let path_obj = Path::new(scan_path);

        if !path_obj.exists() {
            return Err(anyhow::anyhow!("Путь к проекту не существует: {}", scan_path));
        }

        info!("Начало AST-сканирования проекта '{}' (incremental={}) по пути: {}", project_id, incremental, scan_path);

        let known_hashes = if incremental {
            self.get_known_files(project_id)?
        } else {
            std::collections::HashMap::new()
        };

        let scan_res = self.ast_extractor.scan_directory(path_obj, if incremental { Some(&known_hashes) } else { None });
        let now = Utc::now().to_rfc3339();

        let deleted_files: Vec<String> = if incremental {
            known_hashes
                .keys()
                .filter(|k| !scan_res.file_hashes.contains_key(*k))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        if incremental && scan_res.files_scanned == 0 && deleted_files.is_empty() {
            info!("Инкрементальный скан: изменений не обнаружено для проекта '{}'", project_id);
            return Ok(scan_res);
        }

        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        if !incremental {
            tx.execute(
                "DELETE FROM graph_nodes WHERE project_id = ?1 AND provenance = 'ast'",
                params![project_id],
            )?;
            tx.execute(
                "DELETE FROM graph_edges WHERE project_id = ?1 AND provenance = 'ast'",
                params![project_id],
            )?;
            tx.execute(
                "DELETE FROM project_files WHERE project_id = ?1",
                params![project_id],
            )?;
        } else {
            for del_rel in &deleted_files {
                tx.execute(
                    "DELETE FROM graph_nodes WHERE project_id = ?1 AND file_path = ?2",
                    params![project_id, del_rel],
                )?;
                tx.execute(
                    "DELETE FROM project_files WHERE project_id = ?1 AND rel_path = ?2",
                    params![project_id, del_rel],
                )?;
            }

            for rel_path in scan_res.file_meta.keys() {
                tx.execute(
                    "DELETE FROM graph_nodes WHERE project_id = ?1 AND file_path = ?2",
                    params![project_id, rel_path],
                )?;
            }
        }

        // Обновляем project_files
        for (rel, hash) in &scan_res.file_hashes {
            if let Some(&(sz, lines)) = scan_res.file_meta.get(rel) {
                tx.execute(
                    r#"
                    INSERT INTO project_files (project_id, rel_path, sha256, file_size, lines_count, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    ON CONFLICT(project_id, rel_path) DO UPDATE SET
                        sha256 = excluded.sha256,
                        file_size = excluded.file_size,
                        lines_count = excluded.lines_count,
                        updated_at = excluded.updated_at
                    "#,
                    params![project_id, rel, hash, sz as i64, lines as i64, now],
                )?;
            }
        }

        // Вставка узлов
        let mut node_id_to_pk = std::collections::HashMap::new();

        {
            let mut insert_stmt = tx.prepare(
                r#"
                INSERT INTO graph_nodes (node_id, label, node_type, description, project_id, file_path, line_start, line_end, provenance, confidence, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'ast', 1.0, ?9, ?9)
                ON CONFLICT(node_id) DO UPDATE SET
                    label = excluded.label,
                    node_type = excluded.node_type,
                    description = excluded.description,
                    project_id = excluded.project_id,
                    file_path = excluded.file_path,
                    line_start = excluded.line_start,
                    line_end = excluded.line_end,
                    updated_at = excluded.updated_at
                RETURNING id
                "#,
            )?;

            for n in &scan_res.nodes {
                let scoped_node_id = format!("{}:{}", project_id, n.node_id);
                if let Ok(pk) = insert_stmt.query_row(
                    params![
                        scoped_node_id,
                        n.label,
                        n.node_type,
                        n.description,
                        project_id,
                        n.file_path,
                        n.line_start as i64,
                        n.line_end as i64,
                        now
                    ],
                    |r| r.get::<_, i64>(0),
                ) {
                    node_id_to_pk.insert(n.node_id.clone(), pk);
                }
            }
        }

        // Вставка рёбер
        {
            let mut insert_edge_stmt = tx.prepare(
                r#"
                INSERT OR IGNORE INTO graph_edges (source_id, target_id, label, weight, contexts, project_id, provenance, confidence, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'ast', 1.0, ?7, ?7)
                "#,
            )?;

            for e in &scan_res.edges {
                if let (Some(&src_pk), Some(&dst_pk)) = (node_id_to_pk.get(&e.source_node_id), node_id_to_pk.get(&e.target_node_id)) {
                    let ctx_json = serde_json::to_string(&vec![e.context.clone()]).unwrap_or_default();
                    let _ = insert_edge_stmt.execute(params![
                        src_pk,
                        dst_pk,
                        e.label,
                        e.weight,
                        ctx_json,
                        project_id,
                        now
                    ]);
                }
            }
        }

        // Обновляем время сканирования
        tx.execute(
            "UPDATE projects SET last_scanned_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, project_id],
        )?;

        tx.commit()?;

        // Обновляем God Nodes
        let conn_ref = self.conn.lock();
        let _ = crate::graph::GraphAnalytics::update_god_nodes(&conn_ref, project_id);
        drop(conn_ref);

        info!(
            "AST-сканирование '{}' успешно завершено: просканировано {} файлов, найдено {} символов/узлов, {} связей",
            project_id, scan_res.files_scanned, scan_res.nodes.len(), scan_res.edges.len()
        );

        Ok(scan_res)
    }

    /// Гибридный семантический поиск по символам и коду проекта
    pub async fn search_symbols(
        &self,
        project_id: &str,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<ScoredProjectNode>> {
        let words: Vec<String> = query
            .split_whitespace()
            .filter(|w| w.chars().count() >= 2)
            .map(|w| w.to_lowercase())
            .collect();

        let mut scored: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();

        // 1. Лексический поиск
        let candidates = {
            let conn = self.conn.lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, label, description, file_path
                FROM graph_nodes
                WHERE project_id = ?1
                  AND (deleted_at IS NULL OR deleted_at = '')
                "#,
            )?;
            let rows = stmt.query_map(params![project_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            })?;
            let mut list = Vec::new();
            for r in rows.flatten() {
                list.push(r);
            }
            list
        };

        for (id, label, desc_opt, fpath_opt) in candidates {
            let lbl_lower = label.to_lowercase();
            let desc_lower = desc_opt.unwrap_or_default().to_lowercase();
            let fpath_lower = fpath_opt.unwrap_or_default().to_lowercase();

            let mut score = 0.0;
            for w in &words {
                if lbl_lower == *w {
                    score += 50.0;
                } else if lbl_lower.contains(w) {
                    score += 20.0;
                } else if fpath_lower.contains(w) {
                    score += 10.0;
                } else if desc_lower.contains(w) {
                    score += 2.0;
                }
            }

            if score > 0.0 {
                scored.insert(id, score);
            }
        }

        // 2. Векторный семантический поиск
        if let Some(embedder) = &self.embedder
            && let Ok(q_embs) = embedder.embed(&[query.to_string()]).await
                && let Some(q_vec) = q_embs.first() {
                    let cand_blobs = {
                        let conn = self.conn.lock();
                        let mut stmt = conn.prepare(
                            r#"
                            SELECT id, embedding
                            FROM graph_nodes
                            WHERE project_id = ?1 AND embedding IS NOT NULL
                              AND (deleted_at IS NULL OR deleted_at = '')
                            "#,
                        )?;
                        let rows = stmt.query_map(params![project_id], |r| {
                            Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?))
                        })?;
                        let mut list = Vec::new();
                        for r in rows.flatten() {
                            list.push(r);
                        }
                        list
                    };

                    let cand_refs: Vec<(i64, Option<&[u8]>)> = cand_blobs.iter().map(|(id, b)| (*id, Some(b.as_slice()))).collect();
                    for (nid, vscore) in top_k(q_vec, &cand_refs, limit, 0.0) {
                        let entry = scored.entry(nid).or_insert(0.0);
                        *entry += (vscore as f64) * 25.0;
                    }
                }

        let mut sorted_ids: Vec<i64> = scored.keys().copied().collect();
        sorted_ids.sort_by(|a, b| scored[b].partial_cmp(&scored[a]).unwrap_or(std::cmp::Ordering::Equal));
        if sorted_ids.len() > limit {
            sorted_ids.truncate(limit);
        }

        let mut results = Vec::new();
        {
            let conn = self.conn.lock();
            for id in sorted_ids {
                if let Ok(node) = conn.query_row(
                    r#"
                    SELECT id, node_id, label, node_type, file_path, line_start, line_end, description, provenance, is_god_node
                    FROM graph_nodes
                    WHERE id = ?1
                    "#,
                    params![id],
                    |r| {
                        Ok(ScoredProjectNode {
                            id: r.get(0)?,
                            node_id: r.get(1)?,
                            label: r.get(2)?,
                            node_type: r.get(3)?,
                            file_path: r.get(4)?,
                            line_start: r.get(5)?,
                            line_end: r.get(6)?,
                            description: r.get(7)?,
                            score: *scored.get(&id).unwrap_or(&0.0),
                            provenance: r.get(8)?,
                            is_god_node: r.get::<_, i64>(9).unwrap_or(0) == 1,
                        })
                    },
                ) {
                    results.push(node);
                }
            }
        }

        Ok(results)
    }

    /// Статистика проекта
    pub fn get_stats(&self, project_id: &str) -> anyhow::Result<ProjectStats> {
        let project = self.get_project(project_id)?
            .ok_or_else(|| anyhow::anyhow!("Проект с ID '{}' не найден", project_id))?;

        let conn = self.conn.lock();
        let total_nodes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        let ast_nodes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND provenance = 'ast' AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        let total_edges: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        let god_nodes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND is_god_node = 1 AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        Ok(ProjectStats {
            project_id: project.id,
            name: project.name,
            root_path: project.root_path,
            total_nodes: total_nodes as usize,
            ast_nodes: ast_nodes as usize,
            total_edges: total_edges as usize,
            god_nodes: god_nodes as usize,
            last_scanned_at: project.last_scanned_at,
        })
    }
}

/// Ищет корень проекта, поднимаясь вверх от начальной директории к ближайшим маркерам (.git, Cargo.toml, package.json и др.).
pub fn find_project_root(start_dir: &Path) -> PathBuf {
    let mut current = if start_dir.is_file() {
        start_dir.parent().unwrap_or(start_dir).to_path_buf()
    } else {
        start_dir.to_path_buf()
    };

    loop {
        if current.join(".git").exists()
            || current.join("Cargo.toml").exists()
            || current.join("package.json").exists()
            || current.join("pyproject.toml").exists()
            || current.join("requirements.txt").exists()
            || current.join("go.mod").exists()
            || current.join("composer.json").exists()
            || current.join("pubspec.yaml").exists()
            || current.join("pom.xml").exists()
            || current.join("build.gradle").exists()
        {
            return current;
        }

        if let Some(parent) = current.parent() {
            if parent == current {
                break;
            }
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    start_dir.to_path_buf()
}

/// Извлекает название, стек технологий и описание из манифестов репозитория.
pub fn detect_manifest_metadata(root: &Path) -> (String, Vec<String>, Option<String>) {
    let default_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    let mut name = default_name.clone();
    let mut tech_stack = Vec::new();
    let mut description = None;

    // 1. Rust: Cargo.toml
    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        tech_stack.push("rust".to_string());
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name = ") {
                    let n = trimmed.trim_start_matches("name = ").trim().trim_matches('"').trim_matches('\'');
                    if !n.is_empty() {
                        name = n.to_string();
                    }
                } else if trimmed.starts_with("description = ") && description.is_none() {
                    let d = trimmed.trim_start_matches("description = ").trim().trim_matches('"').trim_matches('\'');
                    if !d.is_empty() {
                        description = Some(d.to_string());
                    }
                }
            }
        }
    }

    // 2. Node / TS: package.json
    let package_json = root.join("package.json");
    if package_json.exists() {
        if !tech_stack.contains(&"node".to_string()) {
            tech_stack.push("node".to_string());
        }
        if root.join("tsconfig.json").exists() && !tech_stack.contains(&"typescript".to_string()) {
            tech_stack.push("typescript".to_string());
        }
        if let Ok(content) = std::fs::read_to_string(&package_json)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = v.get("name").and_then(|x| x.as_str())
                    && !n.is_empty() && name == default_name {
                        name = n.to_string();
                    }
                if description.is_none() {
                    description = v.get("description").and_then(|x| x.as_str()).map(|s| s.to_string());
                }
                if v.get("dependencies").and_then(|d| d.get("react")).is_some() && !tech_stack.contains(&"react".to_string()) {
                    tech_stack.push("react".to_string());
                }
            }
    }

    // 3. Python: pyproject.toml / requirements.txt
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() || root.join("setup.py").exists() {
        if !tech_stack.contains(&"python".to_string()) {
            tech_stack.push("python".to_string());
        }
        if let Ok(content) = std::fs::read_to_string(root.join("pyproject.toml")) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name = ") {
                    let n = trimmed.trim_start_matches("name = ").trim().trim_matches('"').trim_matches('\'');
                    if !n.is_empty() && name == default_name {
                        name = n.to_string();
                    }
                }
            }
        }
    }

    // 4. PHP: composer.json
    let composer_json = root.join("composer.json");
    if composer_json.exists() {
        if !tech_stack.contains(&"php".to_string()) {
            tech_stack.push("php".to_string());
        }
        if let Ok(content) = std::fs::read_to_string(&composer_json)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(n) = v.get("name").and_then(|x| x.as_str()) {
                    let short_name = n.split('/').next_back().unwrap_or(n);
                    if !short_name.is_empty() && name == default_name {
                        name = short_name.to_string();
                    }
                }
    }

    // 5. Go: go.mod
    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        if !tech_stack.contains(&"go".to_string()) {
            tech_stack.push("go".to_string());
        }
        if let Ok(content) = std::fs::read_to_string(&go_mod) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("module ") {
                    let mod_path = trimmed.trim_start_matches("module ").trim();
                    let short_name = mod_path.split('/').next_back().unwrap_or(mod_path);
                    if !short_name.is_empty() && name == default_name {
                        name = short_name.to_string();
                    }
                    break;
                }
            }
        }
    }

    // 6. Dart/Flutter: pubspec.yaml
    if root.join("pubspec.yaml").exists() && !tech_stack.contains(&"dart".to_string()) {
        tech_stack.push("dart".to_string());
    }

    // 7. Java: pom.xml / build.gradle
    if (root.join("pom.xml").exists() || root.join("build.gradle").exists()) && !tech_stack.contains(&"java".to_string()) {
        tech_stack.push("java".to_string());
    }

    // README fallback
    if description.is_none()
        && let Ok(content) = std::fs::read_to_string(root.join("README.md")) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    description = Some(trimmed.chars().take(200).collect());
                    break;
                }
            }
        }

    (name, tech_stack, description)
}
