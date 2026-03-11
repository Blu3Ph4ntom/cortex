use crate::model::{
    DependencyDirection, Edge, EdgeKind, ExplainReport, GraphNode, PersistedState, QueryFilter,
    QueryResult, ReferenceReport,
};
use crate::storage::CortexError;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
        let mut nodes = self
            .state
            .graph
            .nodes
            .values()
            .filter(|node| node.kind == crate::model::GraphNodeKind::Symbol)
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
            .cloned()
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(nodes)
    }

    pub fn dependencies(
        &self,
        target: &str,
        direction: DependencyDirection,
        depth: usize,
    ) -> Result<QueryResult, CortexError> {
        let target_id = self.resolve_target_id(target)?;
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([(target_id.clone(), 0usize)]);
        let mut node_ids = BTreeSet::from([target_id.clone()]);
        let mut edge_ids = BTreeSet::new();
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
        let mut impacted = BTreeSet::new();
        let mut supporting_edges = BTreeSet::new();
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
        let summary = format!(
            "{} '{}' has {} incoming edges and {} outgoing edges in revision {}.",
            format!("{:?}", node.kind).to_ascii_lowercase(),
            node.fq_name.clone().unwrap_or_else(|| node.name.clone()),
            incoming,
            outgoing,
            self.state.revision
        );
        Ok(ExplainReport {
            target: node,
            summary,
        })
    }

    fn related<F>(&self, target_id: &str, predicate: F) -> Result<QueryResult, CortexError>
    where
        F: Fn(&Edge, &str) -> bool,
    {
        let mut node_ids = BTreeSet::from([target_id.to_owned()]);
        let mut edge_ids = BTreeSet::new();

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

    fn query_result(&self, node_ids: BTreeSet<String>, edge_ids: BTreeSet<String>) -> QueryResult {
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
