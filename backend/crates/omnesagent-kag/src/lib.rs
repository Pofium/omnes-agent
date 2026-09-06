//! OmnesAgent KAG: Knowledge Augmented Generation, детерминированный AST-граф кода и локальные векторные представления.

pub mod ast;
pub mod embedding;
pub mod graph;
pub mod models;
pub mod project;
pub mod schema;
pub mod vector;

pub use ast::{AstCodeExtractor, AstEdge, AstNode, AstScanResult};
pub use embedding::{default_embedding_provider, EmbeddingProvider, FakeEmbedding};
#[cfg(feature = "candle")]
pub use embedding::LocalBertEmbedding;
pub use graph::{
    AffectedNode, CircularDependency, ComponentMetrics, GodNodeInfo, GraphAnalytics,
    GraphReasonResult, GraphSearchResult, GraphService, GraphStats, ImpactReport, NodeWithNeighbors,
    ProjectReport, RiskLevel,
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
