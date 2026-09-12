//! Сервис графа знаний: дедупликация, поиск и KAG-рассуждение.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
            && let Some(q_vec) = q_embs.first() {
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
        sorted_ids.sort_by(|a, b| scored[b].partial_cmp(&scored[a]).unwrap_or(std::cmp::Ordering::Equal));
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
            let desc = n.description.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
            let loc = n.file_path.as_deref().map(|f| format!(" [{f}]")).unwrap_or_default();
            let god = if n.is_god_node { " [👑 GodNode]" } else { "" };
            facts.push(format!("- `{}` ({}){}{}{}", n.label, n.node_type, loc, desc, god));
        }

        facts.push("\n### Связи и отношения:".to_string());
        for e in found.edges.iter().take(40) {
            facts.push(format!("- `{}` --[{}]--> `{}`", e.source_label, e.label, e.target_label));
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
