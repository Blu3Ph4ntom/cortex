use crate::extractors::DefaultExtractorRegistry;
use crate::model::{
    Edge, EdgeKind, ExtractedRelationKind, GraphNode, GraphNodeKind, GraphSnapshot, HealthReport,
    IndexStats, Language, PersistedState, QueryFilter, SemanticDocument, SymbolKind, display_path,
    external_module_id, file_node_id, normalize_path, repo_node_id, symbol_node_id,
};
use crate::query::QueryEngine;
use crate::storage::{CortexError, SledGraphStore};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub struct RepositorySessionConfig {
    pub repo_path: PathBuf,
    pub store_path: PathBuf,
}

impl RepositorySessionConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        let repo_path = repo_path.into();
        let store_path = repo_path.join(".cortex").join("index");
        Self {
            repo_path,
            store_path,
        }
    }

    pub fn with_store_path(mut self, store_path: impl Into<PathBuf>) -> Self {
        self.store_path = store_path.into();
        self
    }
}

#[derive(Clone)]
pub struct RepositorySession {
    repo_path: PathBuf,
    store: Arc<SledGraphStore>,
    pub(crate) extractors: DefaultExtractorRegistry,
}

impl RepositorySession {
    pub fn open(config: RepositorySessionConfig) -> Result<Self, CortexError> {
        fs::create_dir_all(&config.store_path)?;
        let store = SledGraphStore::open(&config.store_path)?;
        Ok(Self {
            repo_path: display_path(
                &fs::canonicalize(&config.repo_path).unwrap_or(config.repo_path),
            ),
            store: Arc::new(store),
            extractors: DefaultExtractorRegistry::default(),
        })
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        self.extractors.supported_languages()
    }

    pub fn query_engine(&self) -> Result<QueryEngine, CortexError> {
        // Try read-only path first (no sled lock), fall back to write path.
        if let Some(ro) = crate::storage::ReadOnlyStore::load(self.store.as_ref().root())? {
            return Ok(QueryEngine::new(ro.into_state()));
        }
        let state = self.store.load_state()?;
        Ok(QueryEngine::new(state))
    }

    pub(crate) fn store(&self) -> &SledGraphStore {
        self.store.as_ref()
    }

    fn extract_document(&self, path: &Path) -> Result<SemanticDocument, CortexError> {
        let language = Language::from_path(path)
            .ok_or_else(|| CortexError::UnsupportedLanguage(path.to_path_buf()))?;
        let source = fs::read_to_string(path)?;
        let extractor = self.extractors.for_language(language)?;
        extractor.extract(path, &source)
    }
}

pub struct Indexer {
    session: RepositorySession,
}

impl Indexer {
    pub fn new(session: RepositorySession) -> Self {
        Self { session }
    }

    pub fn build_full(&self) -> Result<IndexStats, CortexError> {
        let documents = self.scan_documents()?;
        self.persist_documents(documents, None)
    }

    pub fn refresh_paths(&self, paths: &[PathBuf]) -> Result<IndexStats, CortexError> {
        if paths.iter().any(|path| !path.exists() || path.is_dir()) {
            let state = self.session.store().load_state()?;
            let documents = self.scan_documents()?;
            let revision = state.revision + 1;
            return self.persist_documents(documents, Some(revision));
        }

        let mut state = self.session.store().load_state()?;
        if state.repo_path.is_none() {
            state.repo_path = Some(self.session.repo_path().to_path_buf());
        }

        for path in paths {
            let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            if let Some(language) = Language::from_path(&canonical) {
                let extractor = self.session.extractors.for_language(language)?;
                let source = fs::read_to_string(&canonical)?;
                let document = extractor.extract(&canonical, &source)?;
                state.documents.insert(normalize_path(&canonical), document);
            }
        }

        let revision = state.revision + 1;
        self.persist_documents(state.documents, Some(revision))
    }

    pub fn health(&self) -> Result<HealthReport, CortexError> {
        let state = self.session.store().load_state()?;
        Ok(HealthReport {
            repo_path: self.session.repo_path().to_path_buf(),
            revision: state.revision,
            indexed_files: state.documents.len(),
            supported_languages: self.session.supported_languages(),
        })
    }

    pub fn session(&self) -> &RepositorySession {
        &self.session
    }

    fn scan_documents(&self) -> Result<BTreeMap<String, SemanticDocument>, CortexError> {
        let walker = WalkDir::new(self.session.repo_path())
            .into_iter()
            .filter_entry(|entry| !should_skip(entry));

        let mut documents = BTreeMap::new();
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Err(CortexError::Io(std::io::Error::other(error))),
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if Language::from_path(path).is_none() {
                continue;
            }

            let document = self.session.extract_document(path)?;
            documents.insert(normalize_path(path), document);
        }
        Ok(documents)
    }

    fn persist_documents(
        &self,
        documents: BTreeMap<String, SemanticDocument>,
        revision_override: Option<u64>,
    ) -> Result<IndexStats, CortexError> {
        let revision = revision_override.unwrap_or(1);
        let mut state = PersistedState {
            repo_path: Some(self.session.repo_path().to_path_buf()),
            revision,
            documents,
            graph: GraphSnapshot::default(),
        };
        state.graph = build_graph(self.session.repo_path(), &state.documents);
        self.session.store().save_state(&state)?;
        // Auto-compaction to reclaim space from fragmented sled blobs.
        let bytes_reclaimed = self.session.store().compact().ok();
        Ok(index_stats(&state, bytes_reclaimed))
    }
}

fn should_skip(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git"
            | ".hg"
            | ".svn"
            | ".idea"
            | ".vscode"
            | "node_modules"
            | "dist"
            | "build"
            | "target"
            | ".cortex"
            | "__pycache__"
            | ".venv"
            | "venv"
    )
}

fn index_stats(state: &PersistedState, bytes_reclaimed: Option<u64>) -> IndexStats {
    IndexStats {
        revision: state.revision,
        file_count: state.documents.len(),
        symbol_count: state
            .graph
            .nodes
            .values()
            .filter(|node| node.kind == GraphNodeKind::Symbol)
            .count(),
        edge_count: state.graph.edges.len(),
        bytes_reclaimed,
    }
}

fn build_graph(repo_path: &Path, documents: &BTreeMap<String, SemanticDocument>) -> GraphSnapshot {
    let mut graph = GraphSnapshot::default();
    let repo_id = repo_node_id(repo_path);
    graph.nodes.insert(
        repo_id.clone(),
        GraphNode {
            id: repo_id.clone(),
            kind: GraphNodeKind::Repository,
            language: None,
            symbol_kind: None,
            name: repo_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("repository")
                .to_owned(),
            fq_name: Some(normalize_path(repo_path)),
            path: Some(display_path(repo_path)),
            span: None,
        },
    );

    let mut symbols_by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut symbols_by_file: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for document in documents.values() {
        let file_id = file_node_id(&document.path);
        graph.nodes.insert(
            file_id.clone(),
            GraphNode {
                id: file_id.clone(),
                kind: GraphNodeKind::File,
                language: Some(document.language),
                symbol_kind: None,
                name: document
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("file")
                    .to_owned(),
                fq_name: Some(normalize_path(&document.path)),
                path: Some(display_path(&document.path)),
                span: None,
            },
        );
        insert_edge(
            &mut graph,
            EdgeKind::Contains,
            &repo_id,
            &file_id,
            Some(&document.path),
            "repository contains file",
        );

        let mut file_symbols = BTreeMap::new();
        for symbol in &document.symbols {
            let symbol_id = symbol_node_id(&document.path, &symbol.local_id);
            graph.nodes.insert(
                symbol_id.clone(),
                GraphNode {
                    id: symbol_id.clone(),
                    kind: GraphNodeKind::Symbol,
                    language: Some(document.language),
                    symbol_kind: Some(symbol.kind),
                    name: symbol.name.clone(),
                    fq_name: symbol.fq_name.clone(),
                    path: Some(display_path(&document.path)),
                    span: Some(symbol.span.clone()),
                },
            );
            symbols_by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol_id.clone());
            if let Some(fq_name) = &symbol.fq_name {
                symbols_by_name
                    .entry(fq_name.clone())
                    .or_default()
                    .push(symbol_id.clone());
            }
            file_symbols.insert(symbol.local_id.clone(), symbol_id.clone());

            let owner = symbol
                .parent_local_id
                .as_ref()
                .and_then(|parent| file_symbols.get(parent))
                .cloned()
                .unwrap_or_else(|| file_id.clone());
            insert_edge(
                &mut graph,
                EdgeKind::Defines,
                &owner,
                &symbol_id,
                Some(&document.path),
                "symbol definition",
            );
            insert_edge(
                &mut graph,
                EdgeKind::OwnedBy,
                &symbol_id,
                &file_id,
                Some(&document.path),
                "symbol declared in file",
            );
            if let Some(parent_local_id) = &symbol.parent_local_id
                && let Some(parent_id) = file_symbols.get(parent_local_id)
            {
                insert_edge(
                    &mut graph,
                    EdgeKind::Contains,
                    parent_id,
                    &symbol_id,
                    Some(&document.path),
                    "nested symbol",
                );
            }
        }
        symbols_by_file.insert(normalize_path(&document.path), file_symbols);
    }

    for document in documents.values() {
        let file_id = file_node_id(&document.path);
        let file_symbols = symbols_by_file
            .get(&normalize_path(&document.path))
            .cloned()
            .unwrap_or_default();
        for relation in &document.relations {
            let source_id = relation
                .source_local_id
                .as_ref()
                .and_then(|local_id| file_symbols.get(local_id))
                .cloned()
                .unwrap_or_else(|| file_id.clone());
            let Some(target_id) = resolve_target(
                document,
                &relation.target_name,
                &symbols_by_name,
                &graph.nodes,
            ) else {
                continue;
            };

            let edge_kind = match relation.kind {
                ExtractedRelationKind::Import => EdgeKind::Imports,
                ExtractedRelationKind::Reference => EdgeKind::References,
                ExtractedRelationKind::Call => EdgeKind::Calls,
                ExtractedRelationKind::Inherit => EdgeKind::Inherits,
                ExtractedRelationKind::Implement => EdgeKind::Implements,
            };
            insert_edge(
                &mut graph,
                edge_kind,
                &source_id,
                &target_id,
                Some(&document.path),
                &relation.reason,
            );
            insert_edge(
                &mut graph,
                EdgeKind::DependsOn,
                &source_id,
                &target_id,
                Some(&document.path),
                "derived dependency",
            );
        }
    }

    graph
}

fn resolve_target(
    document: &SemanticDocument,
    target_name: &str,
    symbols_by_name: &BTreeMap<String, Vec<String>>,
    nodes: &BTreeMap<String, GraphNode>,
) -> Option<String> {
    let file_path = normalize_path(&document.path);
    if let Some(symbols) = symbols_by_name.get(target_name) {
        if symbols.len() == 1 {
            return symbols.first().cloned();
        }

        if let Some(local_match) = symbols.iter().find(|symbol_id| {
            nodes
                .get(*symbol_id)
                .and_then(|node| node.path.as_ref())
                .is_some_and(|path| normalize_path(path) == file_path)
        }) {
            return Some(local_match.clone());
        }

        return symbols.first().cloned();
    }

    if looks_like_module(target_name) {
        return Some(external_module_id(target_name));
    }

    None
}

fn looks_like_module(target_name: &str) -> bool {
    target_name.contains('.')
        || target_name.contains('/')
        || target_name.contains("::")
        || target_name.contains('-')
}

fn insert_edge(
    graph: &mut GraphSnapshot,
    kind: EdgeKind,
    from: &str,
    to: &str,
    file_path: Option<&Path>,
    reason: &str,
) {
    if !graph.nodes.contains_key(from) || !graph.nodes.contains_key(to) {
        if kind == EdgeKind::Imports && !graph.nodes.contains_key(to) {
            let module_id = to.to_owned();
            let module_name = module_id.trim_start_matches("module::").to_owned();
            graph.nodes.insert(
                module_id.clone(),
                GraphNode {
                    id: module_id.clone(),
                    kind: GraphNodeKind::Symbol,
                    language: None,
                    symbol_kind: Some(SymbolKind::Module),
                    name: module_name.clone(),
                    fq_name: Some(module_name),
                    path: None,
                    span: None,
                },
            );
        } else if !graph.nodes.contains_key(from) || !graph.nodes.contains_key(to) {
            return;
        }
    }

    let path_string = file_path.map(normalize_path).unwrap_or_default();
    let edge_id = format!("edge::{kind:?}::{from}::{to}::{path_string}");
    graph.edges.entry(edge_id.clone()).or_insert_with(|| Edge {
        id: edge_id,
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
        file_path: file_path.map(display_path),
        reason: reason.to_owned(),
    });
}

pub fn indexed_files(session: &RepositorySession) -> Result<BTreeSet<String>, CortexError> {
    Ok(session
        .query_engine()?
        .state()
        .documents
        .keys()
        .cloned()
        .collect())
}

pub fn find_symbols(
    session: &RepositorySession,
    filter: QueryFilter,
) -> Result<Vec<GraphNode>, CortexError> {
    session.query_engine()?.find_symbol(filter)
}
