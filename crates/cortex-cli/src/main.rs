use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cortex_core::indexer::{Indexer, RepositorySession, RepositorySessionConfig};
use cortex_core::model::{DependencyDirection, QueryFilter, SymbolKind};
use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "cortex")]
#[command(about = "Local-first semantic graph indexing for codebases")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Index {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        store_path: Option<PathBuf>,
    },
    Watch {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        store_path: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        store_path: Option<PathBuf>,
    },
    Export {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        store_path: Option<PathBuf>,
    },
    Query {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        store_path: Option<PathBuf>,
        #[command(subcommand)]
        query: QueryCommand,
    },
}

#[derive(Subcommand, Debug)]
enum QueryCommand {
    FindSymbol {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        fq_name: Option<String>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long, help = "substring match on symbol name (case-insensitive)")]
        name_contains: Option<String>,
        #[arg(long, help = "prefix match on symbol name (case-insensitive)")]
        name_prefix: Option<String>,
        #[arg(long, default_value_t = 50, help = "max results to return")]
        limit: usize,
    },
    Dependencies {
        #[arg(long)]
        target: String,
        #[arg(long, default_value = "outbound")]
        direction: String,
        #[arg(long, default_value_t = 2)]
        depth: usize,
    },
    Callers {
        #[arg(long)]
        target: String,
    },
    Callees {
        #[arg(long)]
        target: String,
    },
    References {
        #[arg(long)]
        target: String,
    },
    Impact {
        #[arg(long)]
        target: String,
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },
    Explain {
        #[arg(long)]
        target: String,
    },
    Search {
        #[arg(long, help = "search query")]
        query: String,
        #[arg(long, default_value_t = 20, help = "max results")]
        limit: usize,
    },
    CrateGraph {},
    Summary {
        #[arg(long, default_value_t = 10, help = "number of top items to show")]
        top: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Index { repo, store_path } => {
            print_json(run_index(&repo, store_path.as_deref())?)?
        }
        Commands::Watch { repo, store_path } => run_watch(&repo, store_path.as_deref())?,
        Commands::Doctor { repo, store_path } => {
            let session = open_session(&repo, store_path.as_deref())?;
            let indexer = Indexer::new(session);
            print_json(indexer.health()?)?;
        }
        Commands::Export { repo, store_path } => {
            let session = open_session(&repo, store_path.as_deref())?;
            let state = session.query_engine()?.state().clone();
            print_json(state.graph)?;
        }
        Commands::Query {
            repo,
            store_path,
            query,
        } => run_query(&repo, store_path.as_deref(), query)?,
    }
    Ok(())
}

fn run_index(repo: &Path, store_path: Option<&Path>) -> Result<cortex_core::model::IndexStats> {
    let session = open_session(repo, store_path)?;
    let indexer = Indexer::new(session);
    indexer.build_full().map_err(Into::into)
}

fn run_query(repo: &Path, store_path: Option<&Path>, query: QueryCommand) -> Result<()> {
    let session = open_session(repo, store_path)?;
    let engine = session.query_engine()?;
    match query {
        QueryCommand::FindSymbol {
            name,
            fq_name,
            kind,
            name_contains,
            name_prefix,
            limit,
        } => {
            let kind = kind.as_deref().and_then(SymbolKind::parse);
            print_json(engine.find_symbol(QueryFilter {
                name,
                fq_name,
                kind,
                name_contains,
                name_prefix,
                limit: Some(limit),
            })?)?;
        }
        QueryCommand::Dependencies {
            target,
            direction,
            depth,
        } => {
            let direction = match direction.as_str() {
                "inbound" => DependencyDirection::Inbound,
                "both" => DependencyDirection::Both,
                _ => DependencyDirection::Outbound,
            };
            print_json(engine.dependencies(&target, direction, depth)?)?;
        }
        QueryCommand::Callers { target } => print_json(engine.callers(&target)?)?,
        QueryCommand::Callees { target } => print_json(engine.callees(&target)?)?,
        QueryCommand::References { target } => print_json(engine.references(&target)?)?,
        QueryCommand::Impact { target, depth } => print_json(engine.impact(&target, depth)?)?,
        QueryCommand::Explain { target } => print_json(engine.explain(&target)?)?,
        QueryCommand::Search { query, limit } => print_json(engine.search(&query, limit))?,
        QueryCommand::CrateGraph {} => print_json(engine.crate_graph()?)?,
        QueryCommand::Summary { top } => print_json(engine.summary(top))?,
    }
    Ok(())
}

fn run_watch(repo: &Path, store_path: Option<&Path>) -> Result<()> {
    let session = open_session(repo, store_path)?;
    let indexer = Indexer::new(session);
    indexer.build_full()?;

    let (tx, rx) = sync_channel::<notify::Result<Event>>(256);
    let mut watcher = recommended_watcher(move |event| {
        let _ = tx.send(event);
    })?;
    watcher.watch(repo, RecursiveMode::Recursive)?;

    let mut pending_paths = BTreeSet::new();
    loop {
        match rx.recv() {
            Ok(Ok(event)) => collect_watch_paths(&mut pending_paths, event),
            Ok(Err(error)) => {
                eprintln!("watch error: {error}");
                continue;
            }
            Err(_) => return Ok(()),
        }

        loop {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(event)) => collect_watch_paths(&mut pending_paths, event),
                Ok(Err(error)) => eprintln!("watch error: {error}"),
                Err(_) => break,
            }
        }

        if pending_paths.is_empty() {
            continue;
        }

        let paths = pending_paths.iter().cloned().collect::<Vec<_>>();
        pending_paths.clear();
        let stats = indexer.refresh_paths(&paths)?;
        print_json(stats)?;
    }
}

fn collect_watch_paths(pending: &mut BTreeSet<PathBuf>, event: Event) {
    for path in event.paths {
        if should_watch_path(&path) {
            pending.insert(path);
        }
    }
}

fn should_watch_path(path: &Path) -> bool {
    if is_ignored_path(path) {
        return false;
    }

    if !path.exists() {
        return true;
    }

    if path.is_file() {
        return cortex_core::model::Language::from_path(path).is_some();
    }

    false
}

fn is_ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
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
    })
}

fn open_session(repo: &Path, store_path: Option<&Path>) -> Result<RepositorySession> {
    let config = match store_path {
        Some(path) => RepositorySessionConfig::new(repo).with_store_path(path),
        None => RepositorySessionConfig::new(repo),
    };
    RepositorySession::open(config)
        .with_context(|| format!("failed to open repository session for {}", repo.display()))
}

fn print_json<T: serde::Serialize>(value: T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
