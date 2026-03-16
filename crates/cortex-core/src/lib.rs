//! Core indexing, storage, and query primitives for Cortex.

pub mod extractors;
pub mod indexer;
pub mod model;
pub mod query;
pub mod storage;

pub use extractors::{DefaultExtractorRegistry, SemanticExtractor};
pub use indexer::{Indexer, RepositorySession, RepositorySessionConfig};
pub use model::{
    CrateEdge, CrateGraph, CrateNode, DependencyDirection, Edge, EdgeKind, ExplainReport,
    GraphNode, GraphNodeKind, GraphSnapshot, HealthReport, ImpactReport, IndexStats, Language,
    QueryFilter, QueryResult, ReferenceReport, RepositorySummary, SearchResult, SymbolKind,
};
pub use storage::{CortexError, ReadOnlyStore, SledGraphStore};
