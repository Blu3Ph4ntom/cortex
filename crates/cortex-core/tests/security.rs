use anyhow::Result;
use cortex_core::model::{GraphNode, GraphNodeKind, PersistedState, Span};
use cortex_core::query::QueryEngine;
use std::fs;
use tempfile::tempdir;

#[test]
fn explain_should_not_read_arbitrary_files() -> Result<()> {
    let temp = tempdir()?;
    let repo_path = temp.path().join("repo");
    fs::create_dir(&repo_path)?;

    let secret_file = temp.path().join("secret.txt");
    fs::write(&secret_file, "sensitive data")?;

    let mut state = PersistedState::default();
    state.repo_path = Some(repo_path.clone());

    let node_id = "test_node".to_string();
    let node = GraphNode {
        id: node_id.clone(),
        kind: GraphNodeKind::Symbol,
        language: None,
        symbol_kind: None,
        name: "test_node".to_string(),
        fq_name: None,
        path: Some(secret_file.clone()),
        span: Some(Span {
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 10,
        }),
    };

    state.graph.nodes.insert(node_id.clone(), node);

    let engine = QueryEngine::new(state);
    let report = engine.explain("test_node")?;

    // Before fix, source_snippet will be Some("sensitive data")
    // After fix, it should be None because it's outside repo_path
    assert!(
        report.source_snippet.is_none(),
        "Should not read file outside repo root"
    );

    Ok(())
}

#[test]
fn explain_should_read_files_inside_repo() -> Result<()> {
    let temp = tempdir()?;
    let repo_path = temp.path().join("repo");
    fs::create_dir(&repo_path)?;

    let internal_file = repo_path.join("main.rs");
    fs::write(&internal_file, "fn main() {}")?;

    let mut state = PersistedState::default();
    state.repo_path = Some(repo_path.clone());

    let node_id = "test_node".to_string();
    let node = GraphNode {
        id: node_id.clone(),
        kind: GraphNodeKind::Symbol,
        language: None,
        symbol_kind: None,
        name: "test_node".to_string(),
        fq_name: None,
        path: Some(internal_file.clone()),
        span: Some(Span {
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 10,
        }),
    };

    state.graph.nodes.insert(node_id.clone(), node);

    let engine = QueryEngine::new(state);
    let report = engine.explain("test_node")?;

    assert_eq!(report.source_snippet, Some("fn main() {}".to_string()));

    Ok(())
}
