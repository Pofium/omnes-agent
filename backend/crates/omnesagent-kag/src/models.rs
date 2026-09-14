//! Модели данных SQLite для KAG и проектной памяти кодинга.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub description: Option<String>,
    pub tech_stack: Option<String>,
    pub active_branch: Option<String>,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: i64,
    pub key: String,
    pub content: String,
    pub category: String,
    pub importance: f64,
    pub source: Option<String>,
    pub namespace: String,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub meta: Option<String>,
    pub embedding: Option<Vec<u8>>,
    pub created_at: String,
    pub updated_at: String,
    pub access_count: i64,
    pub last_accessed: Option<String>,
    pub trust: f64,
    pub last_feedback_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRelationRecord {
    pub source_key: String,
    pub target_key: String,
    pub relation_type: String,
    pub weight: f64,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: i64,
    pub title: Option<String>,
    pub path: Option<String>,
    pub meta: Option<String>,
    pub created_at: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: i64,
    pub doc_id: i64,
    pub ordinal: i64,
    pub text: String,
    pub embedding: Option<Vec<u8>>,
    pub created_at: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeRecord {
    pub id: i64,
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub description: Option<String>,
    pub val: i64,
    pub embedding: Option<Vec<u8>>,
    pub created_at: String,
    pub updated_at: String,
    pub project_id: Option<String>,
    pub file_path: Option<String>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub provenance: Option<String>,
    pub confidence: Option<f64>,
    pub is_god_node: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeRecord {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub label: String,
    pub weight: f64,
    pub contexts: Option<String>,
    pub created_at: String,
    pub project_id: Option<String>,
    pub provenance: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamRunRecord {
    pub id: i64,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub status: Option<String>,
    pub trigger: Option<String>,
    pub phase_log: Option<String>,
    pub stats: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphRunRecord {
    pub id: String,
    pub project_id: String,
    pub feature_slug: String,
    pub goal: String,
    pub status: String,
    pub autonomy: String,
    pub max_iterations_per_task: i64,
    pub max_total_iterations: i64,
    pub budget_tokens: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphIterationRecord {
    pub id: String,
    pub run_id: String,
    pub task_id: String,
    pub n: i64,
    pub git_before: Option<String>,
    pub git_after: Option<String>,
    pub hypothesis: Option<String>,
    pub plan: Option<String>,
    pub result: Option<String>,
    pub tests_summary: Option<String>,
    pub verdict: String,
    pub verdict_source: Option<String>,
    pub ladder_rung: Option<String>,
    pub context_ref: Option<String>,
    pub tokens_used: Option<i64>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphFindingRecord {
    pub id: String,
    pub run_id: Option<String>,
    pub iteration_id: Option<String>,
    pub project_id: Option<String>,
    pub kind: String,
    pub content: String,
    pub symbols: String,
    pub verdict: String,
    pub verdict_source: Option<String>,
    pub embedding: Option<Vec<u8>>,
    pub meta: Option<String>,
    pub created_at: String,
    pub stale_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstChangeRecord {
    pub id: i64,
    pub project_id: String,
    pub run_id: Option<String>,
    pub iteration_id: Option<String>,
    pub path: String,
    pub node_key: String,
    pub label: String,
    pub node_type: String,
    pub change_type: String,
    pub sig_before: Option<String>,
    pub sig_after: Option<String>,
    pub loc_delta: Option<i64>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialize → deserialize → re-serialize and compare the JSON. These
    /// records are the wire shape between the SQLite layer and the MCP
    /// surface, so nullability must stay stable. JSON equality doubles as
    /// content equality here and keeps the production structs derive-lean.
    fn round_trip<T>(value: &T) -> serde_json::Value
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let json = serde_json::to_value(value).expect("serialize");
        let back: T = serde_json::from_value(json.clone()).expect("deserialize");
        let re = serde_json::to_value(&back).expect("re-serialize");
        assert_eq!(re, json, "JSON round trip must preserve the record");
        json
    }

    #[test]
    fn project_record_round_trips_with_null_and_present_options() {
        let rec = ProjectRecord {
            id: "omnes-agent".into(),
            name: "OmnesAgent".into(),
            root_path: "C:/Projects/Omnes-agent".into(),
            description: None,
            tech_stack: Some("rust".into()),
            active_branch: Some("main".into()),
            last_scanned_at: None,
            created_at: "2026-09-11T00:00:00Z".into(),
            updated_at: "2026-09-11T00:00:00Z".into(),
        };
        let json = round_trip(&rec);
        assert_eq!(json["id"], "omnes-agent");
        // Options without skip_serializing_if must serialize as explicit null,
        // not disappear — consumers rely on the stable key set.
        assert!(json.get("description").is_some());
        assert!(json["description"].is_null());
        assert_eq!(json["tech_stack"], "rust");
    }

    #[test]
    fn memory_record_round_trips_including_embedding_blob() {
        let rec = MemoryRecord {
            id: 42,
            key: "arch/hub".into(),
            content: "schema.rs is the god node".into(),
            category: "projects".into(),
            importance: 0.9,
            source: Some("audit".into()),
            namespace: "default".into(),
            agent_id: Some("chief".into()),
            project_id: Some("omnes-agent".into()),
            meta: Some("{\"tags\":[\"arch\"]}".into()),
            embedding: Some(vec![1, 2, 3, 4]),
            created_at: "t0".into(),
            updated_at: "t1".into(),
            access_count: 7,
            last_accessed: Some("t2".into()),
            trust: 0.85,
            last_feedback_at: Some("t3".into()),
        };
        let json = round_trip(&rec);
        assert_eq!(json["embedding"], serde_json::json!([1, 2, 3, 4]));
        assert_eq!(json["access_count"], 7);
        assert_eq!(json["importance"], 0.9);
        assert_eq!(json["trust"], 0.85);
    }

    #[test]
    fn memory_relation_round_trips() {
        let rec = MemoryRelationRecord {
            source_key: "a".into(),
            target_key: "b".into(),
            relation_type: "depends_on".into(),
            weight: 1.5,
            deleted_at: None,
        };
        let json = round_trip(&rec);
        assert_eq!(json["relation_type"], "depends_on");
        assert_eq!(json["weight"], 1.5);
    }

    #[test]
    fn document_and_chunk_records_round_trip() {
        let doc = DocumentRecord {
            id: 1,
            title: Some("PLAN".into()),
            path: Some("PLAN.md".into()),
            meta: None,
            created_at: "t".into(),
            project_id: Some("p".into()),
        };
        let json = round_trip(&doc);
        assert!(json["meta"].is_null());
        assert_eq!(json["title"], "PLAN");

        let chunk = ChunkRecord {
            id: 9,
            doc_id: 1,
            ordinal: 3,
            text: "chunk text".into(),
            embedding: Some(vec![9, 9]),
            created_at: "t".into(),
            project_id: None,
        };
        let json = round_trip(&chunk);
        assert_eq!(json["ordinal"], 3);
        assert!(json["project_id"].is_null());
    }

    #[test]
    fn graph_node_and_edge_records_round_trip_with_provenance_fields() {
        let node = GraphNodeRecord {
            id: 5,
            node_id: "n1".into(),
            label: "schema.rs".into(),
            node_type: "File".into(),
            description: Some("hub".into()),
            val: 1744,
            embedding: None,
            created_at: "t".into(),
            updated_at: "t".into(),
            project_id: Some("omnes-agent".into()),
            file_path: Some("backend/crates/omnesagent-config/src/schema.rs".into()),
            line_start: Some(1),
            line_end: Some(100),
            provenance: Some("ast".into()),
            confidence: Some(1.0),
            is_god_node: Some(true),
        };
        let json = round_trip(&node);
        assert_eq!(json["val"], 1744);
        assert_eq!(json["is_god_node"], true);
        assert_eq!(json["confidence"], 1.0);

        let edge = GraphEdgeRecord {
            id: 6,
            source_id: 5,
            target_id: 7,
            label: "uses".into(),
            weight: 0.75,
            contexts: None,
            created_at: "t".into(),
            project_id: None,
            provenance: Some("ast".into()),
            confidence: None,
        };
        let json = round_trip(&edge);
        assert_eq!(json["label"], "uses");
        assert!(json["contexts"].is_null());
        assert!(json["confidence"].is_null());
    }

    #[test]
    fn dream_run_record_round_trips() {
        let rec = DreamRunRecord {
            id: 2,
            started_at: Some("s".into()),
            finished_at: None,
            status: Some("completed".into()),
            trigger: Some("auto".into()),
            phase_log: Some("[]".into()),
            stats: Some("{}".into()),
        };
        let json = round_trip(&rec);
        assert!(json["finished_at"].is_null());
        assert_eq!(json["status"], "completed");
    }
}
