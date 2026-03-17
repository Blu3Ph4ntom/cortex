use anyhow::Result;
use cortex_core::indexer::{Indexer, RepositorySession, RepositorySessionConfig};
use cortex_core::model::{DependencyDirection, QueryFilter};
use std::fs;
use tempfile::tempdir;

#[test]
fn build_full_should_index_symbols_and_calls() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path();
    fs::write(
        repo.join("main.rs"),
        r#"
fn helper() {}

fn main() {
    helper();
}
"#,
    )?;

    let session = RepositorySession::open(RepositorySessionConfig::new(repo))?;
    let indexer = Indexer::new(session.clone());
    let stats = indexer.build_full()?;
    assert_eq!(stats.file_count, 1);

    let query = session.query_engine()?;
    let symbols = query.find_symbol(QueryFilter {
        name: Some("helper".to_owned()),
        ..QueryFilter::default()
    })?;
    assert_eq!(symbols.len(), 1);

    let callers = query.callers("helper")?;
    assert!(callers.nodes.iter().any(|node| node.name == "main"));
    Ok(())
}

#[test]
fn refresh_paths_should_drop_deleted_files_from_graph() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path();
    let file = repo.join("mod.py");
    fs::write(
        &file,
        r#"
def keep():
    return 1
"#,
    )?;

    let session = RepositorySession::open(RepositorySessionConfig::new(repo))?;
    let indexer = Indexer::new(session.clone());
    indexer.build_full()?;
    fs::remove_file(&file)?;
    let delete_path = repo.join(".").join(file.file_name().expect("test file has a name"));
    let refreshed = indexer.refresh_paths(std::slice::from_ref(&delete_path))?;
    assert_eq!(refreshed.file_count, 0);
    Ok(())
}

#[test]
fn dependencies_should_walk_reverse_and_forward_edges() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path();
    fs::write(
        repo.join("lib.rs"),
        r#"
fn leaf() {}

fn middle() {
    leaf();
}

fn root() {
    middle();
}
"#,
    )?;

    let session = RepositorySession::open(RepositorySessionConfig::new(repo))?;
    let indexer = Indexer::new(session.clone());
    indexer.build_full()?;

    let result = session
        .query_engine()?
        .dependencies("root", DependencyDirection::Both, 3)?;
    assert!(result.nodes.iter().any(|node| node.name == "middle"));
    assert!(result.nodes.iter().any(|node| node.name == "leaf"));
    Ok(())
}

#[test]
fn dependency_direction_should_respect_inbound_and_outbound() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path();
    fs::write(
        repo.join("lib.rs"),
        r#"
fn leaf() {}

fn middle() {
    leaf();
}

fn root() {
    middle();
}
"#,
    )?;

    let session = RepositorySession::open(RepositorySessionConfig::new(repo))?;
    let indexer = Indexer::new(session.clone());
    indexer.build_full()?;
    let query = session.query_engine()?;

    let outbound = query.dependencies("root", DependencyDirection::Outbound, 2)?;
    assert!(outbound.nodes.iter().any(|node| node.name == "middle"));
    assert!(outbound.nodes.iter().any(|node| node.name == "leaf"));

    let inbound = query.dependencies("leaf", DependencyDirection::Inbound, 2)?;
    assert!(inbound.nodes.iter().any(|node| node.name == "middle"));
    assert!(inbound.nodes.iter().any(|node| node.name == "root"));
    Ok(())
}

#[test]
fn rebuild_should_replace_store_directory() -> Result<()> {
    let temp = tempdir()?;
    let repo = temp.path();
    fs::write(
        repo.join("lib.rs"),
        r#"
fn alpha() {}
"#,
    )?;

    let session = RepositorySession::open(RepositorySessionConfig::new(repo))?;
    let indexer = Indexer::new(session.clone());
    indexer.build_full()?;

    let store_root = repo.join(".cortex").join("index");
    let parent = store_root
        .parent()
        .expect("store root should have a parent directory");
    let entries_before = fs::read_dir(&store_root)?.count();

    fs::write(
        repo.join("lib.rs"),
        r#"
fn alpha() {}
fn beta() {}
"#,
    )?;

    indexer.build_full()?;

    let entries_after = fs::read_dir(&store_root)?.count();
    assert!(entries_before > 0);
    assert!(entries_after > 0);

    let mut has_prev = false;
    let mut has_next = false;
    for entry in fs::read_dir(parent)? {
        let name = entry?.file_name();
        if name.to_string_lossy() == "index.prev" {
            has_prev = true;
        }
        if name.to_string_lossy() == "index.next" {
            has_next = true;
        }
    }
    assert!(!has_prev);
    assert!(!has_next);
    Ok(())
}
