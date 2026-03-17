use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    Python,
    Go,
    Rust,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Rust => "rust",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphNodeKind {
    Repository,
    File,
    Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Type,
    Interface,
    Trait,
    Protocol,
    Variable,
    Constant,
    Module,
    Package,
}

impl SymbolKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "function" => Some(Self::Function),
            "method" => Some(Self::Method),
            "class" => Some(Self::Class),
            "type" => Some(Self::Type),
            "interface" => Some(Self::Interface),
            "trait" => Some(Self::Trait),
            "protocol" => Some(Self::Protocol),
            "variable" => Some(Self::Variable),
            "constant" => Some(Self::Constant),
            "module" => Some(Self::Module),
            "package" => Some(Self::Package),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    Contains,
    Defines,
    Imports,
    References,
    Calls,
    Inherits,
    Implements,
    DependsOn,
    OwnedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: GraphNodeKind,
    pub language: Option<Language>,
    pub symbol_kind: Option<SymbolKind>,
    pub name: String,
    pub fq_name: Option<String>,
    pub path: Option<PathBuf>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub kind: EdgeKind,
    pub from: String,
    pub to: String,
    pub file_path: Option<PathBuf>,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub nodes: BTreeMap<String, GraphNode>,
    pub edges: BTreeMap<String, Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedSymbol {
    pub local_id: String,
    pub name: String,
    pub fq_name: Option<String>,
    pub kind: SymbolKind,
    pub span: Span,
    pub parent_local_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractedRelationKind {
    Import,
    Reference,
    Call,
    Inherit,
    Implement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedRelation {
    pub kind: ExtractedRelationKind,
    pub source_local_id: Option<String>,
    pub target_name: String,
    pub span: Span,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDocument {
    pub language: Language,
    pub path: PathBuf,
    pub symbols: Vec<ExtractedSymbol>,
    pub relations: Vec<ExtractedRelation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedState {
    pub repo_path: Option<PathBuf>,
    pub revision: u64,
    pub documents: BTreeMap<String, SemanticDocument>,
    pub graph: GraphSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexStats {
    pub revision: u64,
    pub file_count: usize,
    pub symbol_count: usize,
    pub edge_count: usize,
    /// Bytes reclaimed during auto-compaction (if any).
    pub bytes_reclaimed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub repo_path: PathBuf,
    pub revision: u64,
    pub indexed_files: usize,
    pub supported_languages: Vec<Language>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryFilter {
    pub name: Option<String>,
    pub fq_name: Option<String>,
    pub kind: Option<SymbolKind>,
    /// Case-insensitive substring match on symbol name.
    pub name_contains: Option<String>,
    /// Case-insensitive prefix match on symbol name.
    pub name_prefix: Option<String>,
    /// Maximum number of results to return (default: 50).
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyDirection {
    Outbound,
    Inbound,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceReport {
    pub target: GraphNode,
    pub references: Vec<Edge>,
    pub sources: Vec<GraphNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target: GraphNode,
    pub impacted_nodes: Vec<GraphNode>,
    pub supporting_edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainReport {
    pub target: GraphNode,
    pub summary: String,
    pub incoming_edge_count: usize,
    pub outgoing_edge_count: usize,
    /// Up to 40 lines of the symbol's source definition.
    pub source_snippet: Option<String>,
    /// Up to 5 symbol names that call or reference this symbol.
    pub top_callers: Vec<String>,
    /// Up to 5 symbol names this symbol calls.
    pub top_callees: Vec<String>,
}

/// A scored symbol search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: GraphNode,
    /// 0–100, higher is a better match.
    pub score: u8,
    pub match_reason: String,
}

/// A crate/package node in the workspace dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateNode {
    pub name: String,
    pub path: String,
    pub file_count: usize,
    pub symbol_count: usize,
}

/// A directed dependency between two crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateEdge {
    pub from: String,
    pub to: String,
    /// Number of symbol-level edges that support this dependency.
    pub evidence_count: usize,
}

/// Inter-crate dependency graph for a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateGraph {
    pub crates: Vec<CrateNode>,
    pub edges: Vec<CrateEdge>,
}

/// High-level architectural overview of a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositorySummary {
    pub file_count: usize,
    pub symbol_count: usize,
    pub edge_count: usize,
    /// (language name, file count) pairs sorted by file count descending.
    pub languages: Vec<(String, usize)>,
    /// Up to top_n symbols sorted by inbound edge count descending.
    pub top_referenced: Vec<(GraphNode, usize)>,
    /// Symbols with zero inbound edges and Function kind (likely entry points).
    pub entry_points: Vec<GraphNode>,
    /// (path, symbol_count) pairs for the largest files, descending.
    pub largest_files: Vec<(PathBuf, usize)>,
}

pub fn repo_node_id(repo_path: &Path) -> String {
    format!("repo::{}", normalize_path(repo_path))
}

pub fn file_node_id(path: &Path) -> String {
    format!("file::{}", normalize_path(path))
}

pub fn symbol_node_id(path: &Path, local_id: &str) -> String {
    format!("symbol::{}::{}", normalize_path(path), local_id)
}

pub fn external_module_id(name: &str) -> String {
    format!("module::{}", name)
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

pub fn display_path(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let stripped = raw.strip_prefix("\\\\?\\").unwrap_or(&raw);
    PathBuf::from(stripped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("src/main.rs")), "src/main.rs");
        assert_eq!(normalize_path(Path::new("src\\main.rs")), "src/main.rs");
        assert_eq!(normalize_path(Path::new("Src/Main.RS")), "src/main.rs");
    }

    #[test]
    fn test_display_path() {
        assert_eq!(
            display_path(Path::new("src/main.rs")),
            PathBuf::from("src/main.rs")
        );
        assert_eq!(
            display_path(Path::new("\\\\?\\C:\\project\\src\\main.rs")),
            PathBuf::from("C:\\project\\src\\main.rs")
        );
    }

    #[test]
    fn test_repo_node_id() {
        assert_eq!(repo_node_id(Path::new("project/repo")), "repo::project/repo");
    }

    #[test]
    fn test_file_node_id() {
        assert_eq!(file_node_id(Path::new("src/main.rs")), "file::src/main.rs");
    }

    #[test]
    fn test_symbol_node_id() {
        assert_eq!(
            symbol_node_id(Path::new("src/main.rs"), "foo_bar"),
            "symbol::src/main.rs::foo_bar"
        );
    }

    #[test]
    fn test_external_module_id() {
        assert_eq!(external_module_id("serde"), "module::serde");
    }
}
