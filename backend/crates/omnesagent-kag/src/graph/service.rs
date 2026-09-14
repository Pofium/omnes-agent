//! Сервис графа знаний: дедупликация, поиск и KAG-рассуждение.

use parking_lot::Mutex;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

use crate::embedding::EmbeddingProvider;
use crate::vector::top_k;

pub const REASON_SYSTEM_PROMPT: &str = "\
Ты отвечаешь на вопрос по графу знаний личного агента. Опирайся ТОЛЬКО на \
переданные факты. Верни СТРОГО JSON без markdown:
{\"answer\": \"ответ по-русски\",
 \"confidence\": 0.0,
 \"reasoning_steps\": [\"шаг 1\", \"шаг 2\"],
 \"used_entities\": [\"label сущностей\"],
 \"used_relations\": [\"label отношений\"]}
Если фактов недостаточно — так и скажи в answer, confidence = 0.1.";

pub fn make_node_id(label: &str, node_type: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{label}|{node_type}").as_bytes());
    let hash = hex::encode(hasher.finalize());
    hash[..24].to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWithNeighbors {
    pub id: i64,
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub description: Option<String>,
    pub val: i64,
    pub file_path: Option<String>,
    pub provenance: String,
    pub confidence: f64,
    pub is_god_node: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeWithLabels {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub source_label: String,
    pub target_label: String,
    pub label: String,
    pub weight: f64,
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphSearchResult {
    pub nodes: Vec<NodeWithNeighbors>,
    pub edges: Vec<EdgeWithLabels>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReasonResult {
    pub answer: String,
    pub confidence: f64,
    #[serde(default)]
    pub reasoning_steps: Vec<String>,
    #[serde(default)]
    pub used_entities: Vec<String>,
    #[serde(default)]
    pub used_relations: Vec<String>,
    #[serde(default)]
    pub nodes_used: usize,
    #[serde(default)]
    pub edges_used: usize,
    #[serde(default)]
    pub evidence_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub nodes: i64,
    pub edges: i64,
    pub documents: i64,
    pub chunks: i64,
    pub god_nodes: i64,
}

#[derive(Clone)]
pub struct GraphService {
    conn: Arc<Mutex<Connection>>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl GraphService {
    pub fn new(conn: Arc<Mutex<Connection>>, embedder: Arc<dyn EmbeddingProvider>) -> Self {
        Self { conn, embedder }
    }

    /// Быстрый поиск по графу знаний (лексический + векторный RRF)
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        expand_hops: bool,
        project_id_filter: Option<&str>,
    ) -> anyhow::Result<GraphSearchResult> {
        let words: Vec<String> = query
            .split_whitespace()
            .filter(|w| w.chars().count() >= 2)
            .map(|w| w.to_lowercase())
            .collect();

        let mut scored: HashMap<i64, f64> = HashMap::new();

        // 1. Полнотекстовый / лексический скоринг
        let all_nodes = {
            let conn = self.conn.lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, label, description 
                FROM graph_nodes 
                WHERE (deleted_at IS NULL OR deleted_at = '')
                  AND (?1 IS NULL OR project_id = ?1)
                "#,
            )?;
            let rows = stmt.query_map(params![project_id_filter], |row| {
                let id: i64 = row.get(0)?;
                let label: String = row.get(1)?;
                let desc: Option<String> = row.get(2)?;
                Ok((id, label, desc))
            })?;
            let mut list = Vec::new();
            for r in rows.flatten() {
                list.push(r);
            }
            list
        };

        for (id, label, desc_opt) in all_nodes {
            let label_lower = label.to_lowercase();
            let desc_lower = desc_opt.unwrap_or_default().to_lowercase();
            let mut score = 0.0;
            for w in &words {
                if label_lower == *w {
                    score += 20.0;
                } else if label_lower.contains(w) {
                    score += 10.0;
                } else if desc_lower.contains(w) {
                    score += 2.0;
                }
            }
            if score > 0.0 {
                scored.insert(id, score);
            }
        }

        // 2. Векторный скоринг
        if let Ok(q_embs) = self.embedder.embed(&[query.to_string()]).await
            && let Some(q_vec) = q_embs.first()
        {
            let candidates = {
                let conn = self.conn.lock();
                let mut stmt = conn.prepare(
                    r#"
                        SELECT id, embedding 
                        FROM graph_nodes 
                        WHERE embedding IS NOT NULL 
                          AND (deleted_at IS NULL OR deleted_at = '')
                          AND (?1 IS NULL OR project_id = ?1)
                        "#,
                )?;
                let rows = stmt.query_map(params![project_id_filter], |row| {
                    let id: i64 = row.get(0)?;
                    let blob: Vec<u8> = row.get(1)?;
                    Ok((id, blob))
                })?;
                let mut list = Vec::new();
                for r in rows.flatten() {
                    list.push(r);
                }
                list
            };

            let cand_refs: Vec<(i64, Option<&[u8]>)> = candidates
                .iter()
                .map(|(id, b)| (*id, Some(b.as_slice())))
                .collect();

            for (nid, vscore) in top_k(q_vec, &cand_refs, limit, 0.0) {
                let entry = scored.entry(nid).or_insert(0.0);
                *entry += (vscore as f64) * 10.0;
            }
        }

        let mut sorted_ids: Vec<i64> = scored.keys().copied().collect();
        sorted_ids.sort_by(|a, b| {
            scored[b]
                .partial_cmp(&scored[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if sorted_ids.len() > limit {
            sorted_ids.truncate(limit);
        }

        if sorted_ids.is_empty() {
            return Ok(GraphSearchResult::default());
        }

        // 3. Извлечение найденных узлов
        let mut nodes_map: HashMap<i64, NodeWithNeighbors> = HashMap::new();
        {
            let conn = self.conn.lock();
            for &id in &sorted_ids {
                if let Ok(node) = conn.query_row(
                    r#"
                    SELECT id, node_id, label, node_type, description, val, file_path, provenance, confidence, is_god_node 
                    FROM graph_nodes 
                    WHERE id = ?1
                    "#,
                    params![id],
                    |row| {
                        Ok(NodeWithNeighbors {
                            id: row.get(0)?,
                            node_id: row.get(1)?,
                            label: row.get(2)?,
                            node_type: row.get(3)?,
                            description: row.get(4)?,
                            val: row.get(5)?,
                            file_path: row.get(6)?,
                            provenance: row.get(7)?,
                            confidence: row.get(8)?,
                            is_god_node: row.get::<_, i64>(9).unwrap_or(0) == 1,
                        })
                    },
                ) {
                    nodes_map.insert(id, node);
                }
            }
        }

        // 4. Извлечение рёбер между найденными узлами
        let mut edges = Vec::new();
        let target_ids: Vec<i64> = nodes_map.keys().copied().collect();

        {
            let conn = self.conn.lock();
            for &id in &target_ids {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT e.id, e.source_id, e.target_id, s.label, t.label, e.label, e.weight, e.contexts
                    FROM graph_edges e
                    JOIN graph_nodes s ON s.id = e.source_id
                    JOIN graph_nodes t ON t.id = e.target_id
                    WHERE (e.source_id = ?1 OR e.target_id = ?1) 
                      AND (e.deleted_at IS NULL OR e.deleted_at = '')
                    "#,
                )?;
                let rows = stmt.query_map(params![id], |row| {
                    let ctx_str: Option<String> = row.get(7)?;
                    let contexts = ctx_str
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();
                    Ok(EdgeWithLabels {
                        id: row.get(0)?,
                        source_id: row.get(1)?,
                        target_id: row.get(2)?,
                        source_label: row.get(3)?,
                        target_label: row.get(4)?,
                        label: row.get(5)?,
                        weight: row.get(6)?,
                        contexts,
                    })
                })?;
                for r in rows.flatten() {
                    edges.push(r);
                }
            }
        }

        // 5. 1-hop расширение для связывания смежных узлов
        if expand_hops {
            let conn = self.conn.lock();
            for edge in &edges {
                for nid in [edge.source_id, edge.target_id] {
                    if !nodes_map.contains_key(&nid)
                        && let Ok(neighbor) = conn.query_row(
                            r#"
                            SELECT id, node_id, label, node_type, description, val, file_path, provenance, confidence, is_god_node 
                            FROM graph_nodes 
                            WHERE id = ?1
                            "#,
                            params![nid],
                            |row| {
                                Ok(NodeWithNeighbors {
                                    id: row.get(0)?,
                                    node_id: row.get(1)?,
                                    label: row.get(2)?,
                                    node_type: row.get(3)?,
                                    description: row.get(4)?,
                                    val: row.get(5)?,
                                    file_path: row.get(6)?,
                                    provenance: row.get(7)?,
                                    confidence: row.get(8)?,
                                    is_god_node: row.get::<_, i64>(9).unwrap_or(0) == 1,
                                })
                            },
                        ) {
                            nodes_map.insert(nid, neighbor);
                        }
                }
            }
        }

        Ok(GraphSearchResult {
            nodes: nodes_map.into_values().collect(),
            edges,
        })
    }

    /// Сборка контекста доказательств (evidence facts) для KAG reasoning
    pub async fn collect_evidence_context(
        &self,
        query: &str,
        project_id_filter: Option<&str>,
    ) -> anyhow::Result<(String, usize, usize)> {
        let found = self.search(query, 15, true, project_id_filter).await?;
        if found.nodes.is_empty() {
            return Ok(("В графе знаний нет данных по запросу.".to_string(), 0, 0));
        }

        let mut facts = vec!["### Извлеченные сущности графа знаний:".to_string()];
        for n in &found.nodes {
            let desc = n
                .description
                .as_deref()
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();
            let loc = n
                .file_path
                .as_deref()
                .map(|f| format!(" [{f}]"))
                .unwrap_or_default();
            let god = if n.is_god_node { " [👑 GodNode]" } else { "" };
            facts.push(format!(
                "- `{}` ({}){}{}{}",
                n.label, n.node_type, loc, desc, god
            ));
        }

        facts.push("\n### Связи и отношения:".to_string());
        for e in found.edges.iter().take(40) {
            facts.push(format!(
                "- `{}` --[{}]--> `{}`",
                e.source_label, e.label, e.target_label
            ));
        }

        let nodes_count = found.nodes.len();
        let edges_count = found.edges.len();
        Ok((facts.join("\n"), nodes_count, edges_count))
    }

    /// Статистика графа
    pub fn stats(&self, project_id_filter: Option<&str>) -> anyhow::Result<GraphStats> {
        let conn = self.conn.lock();
        let nodes: i64 = conn.query_row(
            "SELECT count(*) FROM graph_nodes WHERE (deleted_at IS NULL OR deleted_at = '') AND (?1 IS NULL OR project_id = ?1)",
            params![project_id_filter],
            |r| r.get(0),
        )?;
        let edges: i64 = conn.query_row(
            "SELECT count(*) FROM graph_edges WHERE (deleted_at IS NULL OR deleted_at = '') AND (?1 IS NULL OR project_id = ?1)",
            params![project_id_filter],
            |r| r.get(0),
        )?;
        let documents: i64 = conn.query_row(
            "SELECT count(*) FROM documents WHERE (?1 IS NULL OR project_id = ?1)",
            params![project_id_filter],
            |r| r.get(0),
        )?;
        let chunks: i64 = conn.query_row(
            "SELECT count(*) FROM chunks WHERE (?1 IS NULL OR project_id = ?1)",
            params![project_id_filter],
            |r| r.get(0),
        )?;
        let god_nodes: i64 = conn.query_row(
            "SELECT count(*) FROM graph_nodes WHERE is_god_node = 1 AND (deleted_at IS NULL OR deleted_at = '') AND (?1 IS NULL OR project_id = ?1)",
            params![project_id_filter],
            |r| r.get(0),
        )?;

        Ok(GraphStats {
            nodes,
            edges,
            documents,
            chunks,
            god_nodes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::FakeEmbedding;
    use crate::schema::migrate;
    use crate::vector::serialize;

    fn test_service() -> (GraphService, Arc<Mutex<Connection>>) {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let conn = Arc::new(Mutex::new(conn));
        let service = GraphService::new(conn.clone(), Arc::new(FakeEmbedding::new(384)));
        (service, conn)
    }

    /// Live nodes: A(p1, god), B(p1), C(p2); D(p1) is soft-deleted.
    /// Edges: A-[uses]->B (p1), A-[imports]->C (NULL project).
    fn seed_graph(conn: &Connection) {
        conn.execute_batch(
            r#"
            INSERT INTO projects (id, name, root_path, created_at, updated_at)
            VALUES ('p1', 'proj1', 'C:/p1', 't', 't'), ('p2', 'proj2', 'C:/p2', 't', 't');

            INSERT INTO graph_nodes (node_id, label, node_type, description, project_id, is_god_node, created_at, updated_at)
            VALUES ('aaa', 'schema.rs', 'File', 'config schema hub', 'p1', 1, 't', 't');
            INSERT INTO graph_nodes (node_id, label, node_type, description, project_id, created_at, updated_at)
            VALUES ('bbb', 'runtime loop', 'File', 'agent loop', 'p1', 't', 't');
            INSERT INTO graph_nodes (node_id, label, node_type, project_id, created_at, updated_at)
            VALUES ('ccc', 'finance report', 'Document', 'p2', 't', 't');
            INSERT INTO graph_nodes (node_id, label, node_type, project_id, deleted_at, created_at, updated_at)
            VALUES ('ddd', 'schema draft', 'File', 'p1', '2026-01-01', 't', 't');

            INSERT INTO graph_edges (source_id, target_id, label, weight, contexts, project_id, created_at, updated_at)
            VALUES (1, 2, 'uses', 1.5, '["ctx1","ctx2"]', 'p1', 't', 't');
            INSERT INTO graph_edges (source_id, target_id, label, weight, project_id, created_at, updated_at)
            VALUES (1, 3, 'imports', 1.0, NULL, 't', 't');

            INSERT INTO documents (title, project_id, created_at) VALUES ('plan', 'p1', 't');
            INSERT INTO chunks (doc_id, ordinal, text, project_id, created_at) VALUES (1, 0, 'text', 'p1', 't');
            "#,
        )
        .unwrap();
    }

    fn seeded_service() -> GraphService {
        let (service, conn) = test_service();
        seed_graph(&conn.lock());
        service
    }

    #[test]
    fn make_node_id_is_deterministic_24_hex_and_type_sensitive() {
        let a = make_node_id("schema.rs", "File");
        let b = make_node_id("schema.rs", "File");
        let c = make_node_id("schema.rs", "Function");
        assert_eq!(a, b, "same label+type must hash identically");
        assert_ne!(a, c, "node_type participates in the hash");
        assert_eq!(a.len(), 24);
        assert!(a.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn stats_counts_live_rows_and_respects_project_filter() {
        let service = seeded_service();

        let all = service.stats(None).unwrap();
        assert_eq!(all.nodes, 3, "soft-deleted node must not count");
        assert_eq!(all.edges, 2);
        assert_eq!(all.documents, 1);
        assert_eq!(all.chunks, 1);
        assert_eq!(all.god_nodes, 1);

        let p2 = service.stats(Some("p2")).unwrap();
        assert_eq!(p2.nodes, 1);
        assert_eq!(
            p2.edges, 0,
            "edge with NULL project_id belongs to no project"
        );
        assert_eq!(p2.documents, 0);
        assert_eq!(p2.god_nodes, 0);
    }

    #[tokio::test]
    async fn search_finds_lexical_matches_and_hides_deleted_and_foreign_projects() {
        let service = seeded_service();

        let hit = service.search("schema", 10, false, None).await.unwrap();
        assert!(hit.nodes.iter().any(|n| n.label == "schema.rs"));
        assert!(
            hit.nodes.iter().all(|n| n.label != "schema draft"),
            "soft-deleted node must not surface"
        );
        let god = hit.nodes.iter().find(|n| n.label == "schema.rs").unwrap();
        assert!(god.is_god_node);
        assert_eq!(god.provenance, "manual");
        assert!((god.confidence - 1.0).abs() < 1e-9);

        let description_hit = service.search("hub", 10, false, None).await.unwrap();
        assert!(
            description_hit.nodes.iter().any(|n| n.label == "schema.rs"),
            "description-only match must be found"
        );

        let foreign = service
            .search("schema", 10, false, Some("p2"))
            .await
            .unwrap();
        assert!(foreign.nodes.is_empty(), "project filter must apply");
    }

    #[tokio::test]
    async fn search_expand_hops_pulls_edge_neighbors_into_result() {
        let service = seeded_service();

        let flat = service.search("schema.rs", 10, false, None).await.unwrap();
        assert!(flat.nodes.iter().any(|n| n.label == "schema.rs"));
        assert!(!flat.nodes.iter().any(|n| n.label == "runtime loop"));

        let expanded = service.search("schema.rs", 10, true, None).await.unwrap();
        let labels = labels_of(&expanded);
        assert!(labels.contains(&"schema.rs".to_string()));
        assert!(labels.contains(&"runtime loop".to_string()));
        assert!(
            labels.contains(&"finance report".to_string()),
            "1-hop must pull A-C neighbor"
        );
        assert_eq!(
            expanded.edges.len(),
            2,
            "both edges of the hub are returned"
        );
        let uses = expanded
            .edges
            .iter()
            .find(|e| e.label == "uses")
            .expect("uses edge present");
        assert_eq!(uses_contexts(uses), &["ctx1", "ctx2"]);
        assert_eq!(uses.source_label, "schema.rs");
        assert_eq!(uses.target_label, "runtime loop");
    }

    #[tokio::test]
    async fn search_returns_default_when_graph_has_no_match() {
        let service = seeded_service();
        let none = service
            .search("totally-unknown-term", 10, true, None)
            .await
            .unwrap();
        assert!(none.nodes.is_empty());
        assert!(none.edges.is_empty());
    }

    #[tokio::test]
    async fn search_vector_scoring_finds_node_without_lexical_overlap() {
        let (service, conn) = test_service();
        let embedder = FakeEmbedding::new(384);
        let qvec = embedder
            .embed(&["zzquantumzz".to_string()])
            .await
            .unwrap()
            .pop()
            .unwrap();
        {
            let conn = conn.lock();
            conn.execute(
                "INSERT INTO graph_nodes (node_id, label, node_type, project_id, embedding, created_at, updated_at)
                 VALUES ('fff', 'xq9', 'File', NULL, ?1, 't', 't')",
                params![serialize(&qvec)],
            )
            .unwrap();
        }

        let hit = service
            .search("zzquantumzz", 10, false, None)
            .await
            .unwrap();
        assert!(
            hit.nodes.iter().any(|n| n.label == "xq9"),
            "vector-only match must surface via embedding scoring"
        );
    }

    #[tokio::test]
    async fn collect_evidence_context_formats_facts_and_reports_counts() {
        let service = seeded_service();

        let (context, nodes_used, edges_used) = service
            .collect_evidence_context("schema.rs", None)
            .await
            .unwrap();
        assert!(nodes_used >= 1);
        assert!(edges_used >= 1);
        assert!(context.contains("Извлеченные сущности"));
        assert!(context.contains("schema.rs"));
        assert!(context.contains("GodNode"), "god node must be flagged");
        assert!(context.contains("--[uses]-->"));

        let empty_service = test_service().0;
        let (context, nodes, edges) = empty_service
            .collect_evidence_context("anything", None)
            .await
            .unwrap();
        assert_eq!((nodes, edges), (0, 0));
        assert!(context.contains("нет данных"));
    }

    // -- helpers ---------------------------------------------------------

    fn labels_of(result: &GraphSearchResult) -> Vec<String> {
        result.nodes.iter().map(|n| n.label.clone()).collect()
    }

    fn uses_contexts(edge: &EdgeWithLabels) -> &[String] {
        &edge.contexts
    }
}
