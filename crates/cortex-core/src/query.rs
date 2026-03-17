use crate::model::{
    CrateEdge, CrateGraph, CrateNode, DependencyDirection, Edge, EdgeKind, ExplainReport,
    GraphNode, GraphNodeKind, PersistedState, QueryFilter, QueryResult, ReferenceReport,
    RepositorySummary, SearchResult, SymbolKind, normalize_path,
};
use crate::storage::CortexError;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct QueryEngine {
    state: PersistedState,
}

impl QueryEngine {
    pub fn new(state: PersistedState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &PersistedState {
        &self.state
    }

    pub fn find_symbol(&self, filter: QueryFilter) -> Result<Vec<GraphNode>, CortexError> {
        let limit = filter.limit.unwrap_or(50);
        let mut nodes = self
            .state
            .graph
            .nodes
            .values()
            .filter(|node| node.kind == GraphNodeKind::Symbol)
            .filter(|node| match &filter.name {
                Some(name) => &node.name == name,
                None => true,
            })
            .filter(|node| match &filter.fq_name {
                Some(fq_name) => node.fq_name.as_ref() == Some(fq_name),
                None => true,
            })
            .filter(|node| match filter.kind {
                Some(kind) => node.symbol_kind == Some(kind),
                None => true,
            })
            .filter(|node| match &filter.name_contains {
                Some(pat) => node
                    .name
                    .to_ascii_lowercase()
                    .contains(&pat.to_ascii_lowercase()),
                None => true,
            })
            .filter(|node| match &filter.name_prefix {
                Some(prefix) => node
                    .name
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase()),
                None => true,
            })
            .cloned()
            .collect::<Vec<_>>();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        nodes.truncate(limit);
        Ok(nodes)
    }

    pub fn dependencies(
        &self,
        target: &str,
        direction: DependencyDirection,
        depth: usize,
    ) -> Result<QueryResult, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([(target_id.clone(), 0usize)]);
        let mut node_ids = HashSet::from([target_id.clone()]);
        let mut edge_ids = HashSet::new();
        while let Some((current, level)) = queue.pop_front() {
            if level >= depth || !visited.insert(current.clone()) {
                continue;
            }

            for (edge, next) in dependency_neighbors(&self.state.graph.edges, &current, direction) {
                edge_ids.insert(edge.id.clone());
                if node_ids.insert(next.to_owned()) {
                    queue.push_back((next.to_owned(), level + 1));
                }
            }
        }

        Ok(self.query_result(node_ids, edge_ids))
    }

    pub fn callers(&self, target: &str) -> Result<QueryResult, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        self.related(&target_id, |edge, current| {
            edge.kind == EdgeKind::Calls && edge.to == current
        })
    }

    pub fn callees(&self, target: &str) -> Result<QueryResult, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        self.related(&target_id, |edge, current| {
            edge.kind == EdgeKind::Calls && edge.from == current
        })
    }

    pub fn references(&self, target: &str) -> Result<ReferenceReport, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        let target_node = self.node(&target_id)?.clone();
        let references = self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| {
                matches!(
                    edge.kind,
                    EdgeKind::References | EdgeKind::Calls | EdgeKind::Imports
                ) && edge.to == target_id
            })
            .cloned()
            .collect::<Vec<_>>();
        let sources = references
            .iter()
            .filter_map(|edge| self.state.graph.nodes.get(&edge.from))
            .cloned()
            .collect::<Vec<_>>();
        Ok(ReferenceReport {
            target: target_node,
            references,
            sources,
        })
    }

    pub fn impact(
        &self,
        target: &str,
        depth: usize,
    ) -> Result<crate::model::ImpactReport, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        let mut impacted = HashSet::new();
        let mut supporting_edges = HashSet::new();
        let mut queue = VecDeque::from([(target_id.clone(), 0usize)]);

        while let Some((current, level)) = queue.pop_front() {
            if level >= depth {
                continue;
            }

            for edge in self.state.graph.edges.values().filter(|edge| {
                matches!(
                    edge.kind,
                    EdgeKind::DependsOn
                        | EdgeKind::Calls
                        | EdgeKind::References
                        | EdgeKind::Imports
                ) && edge.to == current
            }) {
                supporting_edges.insert(edge.id.clone());
                if impacted.insert(edge.from.clone()) {
                    queue.push_back((edge.from.clone(), level + 1));
                }
            }
        }

        Ok(crate::model::ImpactReport {
            target: self.node(&target_id)?.clone(),
            impacted_nodes: impacted
                .iter()
                .filter_map(|id| self.state.graph.nodes.get(id))
                .cloned()
                .collect(),
            supporting_edges: supporting_edges
                .iter()
                .filter_map(|id| self.state.graph.edges.get(id))
                .cloned()
                .collect(),
        })
    }

    pub fn explain(&self, target: &str) -> Result<ExplainReport, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        let node = self.node(&target_id)?.clone();

        let incoming = self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| edge.to == target_id)
            .count();
        let outgoing = self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| edge.from == target_id)
            .count();

        // Top callers: nodes that Call or Reference this symbol (up to 5, unique names).
        let mut caller_names: Vec<String> = self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| {
                matches!(edge.kind, EdgeKind::Calls | EdgeKind::References) && edge.to == target_id
            })
            .filter_map(|edge| self.state.graph.nodes.get(&edge.from))
            .map(|n| n.name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        caller_names.sort();
        caller_names.truncate(5);

        // Top callees: nodes that this symbol Calls (up to 5, unique names).
        let mut callee_names: Vec<String> = self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| edge.kind == EdgeKind::Calls && edge.from == target_id)
            .filter_map(|edge| self.state.graph.nodes.get(&edge.to))
            .map(|n| n.name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        callee_names.sort();
        callee_names.truncate(5);

        // Source snippet: read lines start_line..=end_line, cap at 40 lines.
        let source_snippet = node
            .path
            .as_ref()
            .zip(node.span.as_ref())
            .and_then(|(path, span)| {
                let source = std::fs::read_to_string(path).ok()?;
                let start = span.start_line.saturating_sub(1);
                let count = (span.end_line - span.start_line + 1).min(40);
                let snippet: String = source
                    .lines()
                    .skip(start)
                    .take(count)
                    .collect::<Vec<_>>()
                    .join("\n");
                if snippet.is_empty() {
                    None
                } else {
                    Some(snippet)
                }
            });

        let kind_label = node
            .symbol_kind
            .map(|k| format!("{k:?}"))
            .unwrap_or_else(|| "Symbol".to_owned());
        let location = node
            .path
            .as_ref()
            .zip(node.span.as_ref())
            .map(|(p, s)| format!("{}:{}", p.display(), s.start_line))
            .unwrap_or_else(|| "unknown location".to_owned());

        let summary = format!(
            "{} [{kind_label}] — {incoming} callers, {outgoing} dependencies. Defined at {location}.",
            node.fq_name.as_deref().unwrap_or(&node.name),
        );

        Ok(ExplainReport {
            target: node,
            summary,
            incoming_edge_count: incoming,
            outgoing_edge_count: outgoing,
            source_snippet,
            top_callers: caller_names,
            top_callees: callee_names,
        })
    }

    /// Score-ranked symbol search using name, path, and kind heuristics.
    /// No embeddings required — pure text scoring.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let q = query.to_ascii_lowercase();

        let mut results: Vec<SearchResult> = self
            .state
            .graph
            .nodes
            .values()
            .filter(|node| node.kind == GraphNodeKind::Symbol)
            .filter_map(|node| {
                let mut score: i32 = 0;
                let mut reasons: Vec<&str> = Vec::new();

                let name_lower = node.name.to_ascii_lowercase();
                if name_lower.contains(&q) {
                    score += 50;
                    reasons.push("name match");
                }

                if let Some(path) = &node.path
                    && normalize_path(path).contains(&q)
                {
                    score += 30;
                    reasons.push("file path match");
                }

                if let Some(fq) = &node.fq_name
                    && fq.to_ascii_lowercase().contains(&q)
                    && !reasons.contains(&"name match")
                {
                    score += 20;
                    reasons.push("qualified name match");
                }

                match node.symbol_kind {
                    Some(SymbolKind::Type)
                    | Some(SymbolKind::Trait)
                    | Some(SymbolKind::Interface)
                    | Some(SymbolKind::Class) => score += 10,
                    Some(SymbolKind::Variable) | Some(SymbolKind::Constant) => score -= 10,
                    _ => {}
                }

                if score <= 0 {
                    return None;
                }

                Some(SearchResult {
                    node: node.clone(),
                    score: score.clamp(0, 100) as u8,
                    match_reason: reasons.join(", "),
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score).then(a.node.name.cmp(&b.node.name)));
        results.truncate(limit);
        results
    }

    /// Build an inter-crate dependency graph by scanning for Cargo.toml files
    /// and grouping symbol-level edges that cross crate boundaries.
    pub fn crate_graph(&self) -> Result<CrateGraph, CortexError> {
        let repo_path = match &self.state.repo_path {
            Some(p) => p.clone(),
            None => {
                return Ok(CrateGraph {
                    crates: vec![],
                    edges: vec![],
                });
            }
        };

        // Discover crates by finding Cargo.toml files.
        let mut crate_dirs: Vec<(String, PathBuf)> = Vec::new();
        for entry in WalkDir::new(&repo_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() != "Cargo.toml" {
                continue;
            }
            let norm = normalize_path(entry.path());
            if norm.contains("/target/") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            // Skip pure workspace manifests (no [package] section).
            if content.contains("[workspace]") && !content.contains("[package]") {
                continue;
            }
            let dir = entry.path().parent().unwrap_or(entry.path()).to_path_buf();
            let name = extract_cargo_name(&content).unwrap_or_else(|| {
                dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_owned()
            });
            crate_dirs.push((name, dir));
        }

        // Map a file path to the crate it belongs to (longest prefix wins).
        let find_crate = |path: &std::path::Path| -> Option<String> {
            let norm = normalize_path(path);
            crate_dirs
                .iter()
                .filter(|(_, dir)| norm.starts_with(&normalize_path(dir)))
                .max_by_key(|(_, dir)| normalize_path(dir).len())
                .map(|(name, _)| name.clone())
        };

        // Count files and symbols per crate.
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        let mut symbol_counts: HashMap<String, usize> = HashMap::new();

        for node in self.state.graph.nodes.values() {
            let Some(path) = &node.path else { continue };
            let Some(crate_name) = find_crate(path) else {
                continue;
            };
            match node.kind {
                GraphNodeKind::File => {
                    *file_counts.entry(crate_name).or_default() += 1;
                }
                GraphNodeKind::Symbol => {
                    *symbol_counts.entry(crate_name).or_default() += 1;
                }
                GraphNodeKind::Repository => {}
            }
        }

        let crates: Vec<CrateNode> = crate_dirs
            .iter()
            .map(|(name, dir)| CrateNode {
                name: name.clone(),
                path: normalize_path(dir),
                file_count: file_counts.get(name).copied().unwrap_or(0),
                symbol_count: symbol_counts.get(name).copied().unwrap_or(0),
            })
            .collect();

        // Accumulate cross-crate dependency evidence.
        let mut edge_counts: HashMap<(String, String), usize> = HashMap::new();
        for edge in self.state.graph.edges.values() {
            if edge.kind != EdgeKind::DependsOn {
                continue;
            }
            let from_path = self
                .state
                .graph
                .nodes
                .get(&edge.from)
                .and_then(|n| n.path.as_ref());
            let to_path = self
                .state
                .graph
                .nodes
                .get(&edge.to)
                .and_then(|n| n.path.as_ref());
            if let (Some(fp), Some(tp)) = (from_path, to_path)
                && let (Some(fc), Some(tc)) = (find_crate(fp), find_crate(tp))
                && fc != tc
            {
                *edge_counts.entry((fc, tc)).or_default() += 1;
            }
        }

        let edges: Vec<CrateEdge> = edge_counts
            .into_iter()
            .map(|((from, to), count)| CrateEdge {
                from,
                to,
                evidence_count: count,
            })
            .collect();

        Ok(CrateGraph { crates, edges })
    }

    /// High-level architectural overview of the indexed repository.
    pub fn summary(&self, top_n: usize) -> RepositorySummary {
        let file_count = self
            .state
            .graph
            .nodes
            .values()
            .filter(|n| n.kind == GraphNodeKind::File)
            .count();
        let symbol_count = self
            .state
            .graph
            .nodes
            .values()
            .filter(|n| n.kind == GraphNodeKind::Symbol)
            .count();
        let edge_count = self.state.graph.edges.len();

        // Language breakdown.
        let mut lang_counts: HashMap<String, usize> = HashMap::new();
        for node in self.state.graph.nodes.values() {
            if node.kind == GraphNodeKind::File
                && let Some(lang) = node.language
            {
                *lang_counts.entry(lang.as_str().to_owned()).or_default() += 1;
            }
        }
        let mut languages: Vec<(String, usize)> = lang_counts.into_iter().collect();
        languages.sort_by(|a, b| b.1.cmp(&a.1));

        // Inbound edge count per symbol node.
        let mut inbound: HashMap<&str, usize> = HashMap::new();
        for edge in self.state.graph.edges.values() {
            *inbound.entry(edge.to.as_str()).or_default() += 1;
        }

        let mut symbol_nodes: Vec<&GraphNode> = self
            .state
            .graph
            .nodes
            .values()
            .filter(|n| n.kind == GraphNodeKind::Symbol)
            .collect();

        // Top referenced.
        symbol_nodes.sort_by(|a, b| {
            let ia = inbound.get(a.id.as_str()).copied().unwrap_or(0);
            let ib = inbound.get(b.id.as_str()).copied().unwrap_or(0);
            ib.cmp(&ia).then(a.name.cmp(&b.name))
        });
        let top_referenced: Vec<(GraphNode, usize)> = symbol_nodes
            .iter()
            .take(top_n)
            .map(|n| {
                let count = inbound.get(n.id.as_str()).copied().unwrap_or(0);
                ((*n).clone(), count)
            })
            .collect();

        // Entry points: Function symbols with zero inbound edges.
        let entry_points: Vec<GraphNode> = symbol_nodes
            .iter()
            .filter(|n| {
                n.symbol_kind == Some(SymbolKind::Function)
                    && inbound.get(n.id.as_str()).copied().unwrap_or(0) == 0
            })
            .take(top_n)
            .map(|n| (*n).clone())
            .collect();

        // Largest files by symbol count.
        let mut file_symbol_counts: HashMap<PathBuf, usize> = HashMap::new();
        for node in self.state.graph.nodes.values() {
            if node.kind == GraphNodeKind::Symbol
                && let Some(path) = &node.path
            {
                *file_symbol_counts.entry(path.clone()).or_default() += 1;
            }
        }
        let mut largest_files: Vec<(PathBuf, usize)> = file_symbol_counts.into_iter().collect();
        largest_files.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        largest_files.truncate(top_n);

        RepositorySummary {
            file_count,
            symbol_count,
            edge_count,
            languages,
            top_referenced,
            entry_points,
            largest_files,
        }
    }

    fn related<F>(&self, target_id: &str, predicate: F) -> Result<QueryResult, CortexError>
    where
        F: Fn(&Edge, &str) -> bool,
    {
        let mut node_ids = HashSet::from([target_id.to_owned()]);
        let mut edge_ids = HashSet::new();

        for edge in self
            .state
            .graph
            .edges
            .values()
            .filter(|edge| predicate(edge, target_id))
        {
            node_ids.insert(edge.from.clone());
            node_ids.insert(edge.to.clone());
            edge_ids.insert(edge.id.clone());
        }

        Ok(self.query_result(node_ids, edge_ids))
    }

    fn node(&self, id: &str) -> Result<&GraphNode, CortexError> {
        self.state
            .graph
            .nodes
            .get(id)
            .ok_or_else(|| CortexError::NotFound(id.to_owned()))
    }

    fn resolve_target_id(&self, target: &str) -> Result<String, CortexError> {
        if self.state.graph.nodes.contains_key(target) {
            return Ok(target.to_owned());
        }

        if let Some((id, _)) = self
            .state
            .graph
            .nodes
            .iter()
            .find(|(_, node)| node.name == target || node.fq_name.as_deref() == Some(target))
        {
            return Ok(id.clone());
        }

        Err(CortexError::NotFound(target.to_owned()))
    }

    fn query_result(&self, node_ids: HashSet<String>, edge_ids: HashSet<String>) -> QueryResult {
        QueryResult {
            nodes: node_ids
                .iter()
                .filter_map(|id| self.state.graph.nodes.get(id))
                .cloned()
                .collect(),
            edges: edge_ids
                .iter()
                .filter_map(|id| self.state.graph.edges.get(id))
                .cloned()
                .collect(),
        }
    }
}

fn dependency_neighbors<'a>(
    edges: &'a BTreeMap<String, Edge>,
    current: &str,
    direction: DependencyDirection,
) -> Vec<(&'a Edge, &'a str)> {
    edges
        .values()
        .filter(|edge| edge.kind == EdgeKind::DependsOn)
        .filter_map(|edge| match direction {
            DependencyDirection::Outbound if edge.from == current => Some((edge, edge.to.as_str())),
            DependencyDirection::Inbound if edge.to == current => Some((edge, edge.from.as_str())),
            DependencyDirection::Both if edge.from == current => Some((edge, edge.to.as_str())),
            DependencyDirection::Both if edge.to == current => Some((edge, edge.from.as_str())),
            _ => None,
        })
        .collect()
}

fn extract_cargo_name(content: &str) -> Option<String> {
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_package = false;
        }
        if in_package
            && trimmed.starts_with("name")
            && let Some(val) = trimmed.split_once('=').map(|x| x.1)
        {
            return Some(val.trim().trim_matches('"').to_owned());
        }
    }
    None
}
