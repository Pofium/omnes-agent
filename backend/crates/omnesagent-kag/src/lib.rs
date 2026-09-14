//! OmnesAgent KAG: Knowledge Augmented Generation, детерминированный AST-граф кода и локальные векторные представления.

pub mod ast;
pub mod bench;
pub mod embedding;
pub mod graph;
pub mod models;
pub mod project;
pub mod schema;
pub mod vector;

pub use ast::{AstCodeExtractor, AstEdge, AstNode, AstScanResult};
pub use bench::{
    evaluate_gate_decision, evaluate_ranked_results, is_degradation, mrr, parse_golden_jsonl,
    percentile, recall_at_k, BenchSummary, GateVerdict, GoldenCase, MRR_DROP_LIMIT, QUICK_CASES,
    RECALL_DROP_LIMIT,
};
pub use embedding::{default_embedding_provider, EmbeddingProvider, FakeEmbedding};
#[cfg(feature = "candle")]
pub use embedding::LocalBertEmbedding;
pub use graph::{
    build_repo_map, latest_hint, personalized_pagerank, warn_block, AffectedNode, BlastHint,
    CallLink, CallPathResult, CircularDependency, ComponentMetrics, DeadSymbol, Dir, GodNodeInfo,
    GraphAnalytics, GraphReasonResult, GraphSearchResult, GraphService, GraphStats, ImpactReport,
    NodeWithNeighbors, PprEdge, PprGraph, ProjectReport, RiskLevel, SymbolRef, Zone, ZonesReport,
};
pub use models::{
    ChunkRecord, DocumentRecord, DreamRunRecord, GraphEdgeRecord, GraphNodeRecord,
    MemoryRecord, MemoryRelationRecord, ProjectRecord,
};
pub use project::{
    detect_manifest_metadata, find_project_root, ProjectService, ProjectStats, ProjectWatcher,
    ScoredProjectNode,
};
pub use schema::{migrate, SCHEMA_VERSION};
pub use vector::{cosine, deserialize, normalize, rrf_merge, serialize, top_k, RankedItem};
