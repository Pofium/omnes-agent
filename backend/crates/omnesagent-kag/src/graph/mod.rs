//! Сервис графа знаний (KAG) и архитектурной аналитики.

pub mod analytics;
pub mod blast;
pub mod callpath;
pub mod communities;
pub mod pagerank;
pub mod repomap;
pub mod service;

pub use analytics::{
    AffectedNode, CircularDependency, ComponentMetrics, GodNodeInfo, GraphAnalytics, ImpactReport,
    ProjectReport, RiskLevel,
};
pub use blast::{latest_hint, warn_block, BlastHint};
pub use callpath::{
    call_path, dead_code, format_dead_code, format_links, format_path, neighbors, resolve_symbol,
    CallLink, CallPathResult, DeadSymbol, Dir, SymbolRef,
};
pub use communities::{detect_zones, format_zones, mentioned_symbols, Zone, ZonesReport};
pub use pagerank::{
    normalize_entity, parse_ppr_weights, personalized_pagerank, weight_for, PprEdge, PprGraph,
    PprWeights, DEFAULT_PPR_WEIGHTS, PPR_UNKNOWN_WEIGHT,
};
pub use repomap::build_repo_map;
pub use service::{
    make_node_id, EdgeWithLabels, GraphReasonResult, GraphSearchResult, GraphService, GraphStats,
    NodeWithNeighbors,
};
