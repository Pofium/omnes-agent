//! Сервис графа знаний (KAG) и архитектурной аналитики.

pub mod analytics;
pub mod service;

pub use analytics::{
    AffectedNode, CircularDependency, ComponentMetrics, GodNodeInfo, GraphAnalytics, ImpactReport,
    ProjectReport, RiskLevel,
};
pub use service::{
    make_node_id, EdgeWithLabels, GraphReasonResult, GraphSearchResult, GraphService, GraphStats,
    NodeWithNeighbors,
};
