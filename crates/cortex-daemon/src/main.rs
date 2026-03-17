use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    response::Response,
    routing::{get, post},
};
use clap::Parser;
use cortex_core::indexer::{Indexer, RepositorySession, RepositorySessionConfig};
use cortex_core::model::{DependencyDirection, QueryFilter, SymbolKind};
use cortex_core::query::QueryEngine;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    repo: PathBuf,
    #[arg(long)]
    store_path: Option<PathBuf>,
    #[arg(long, default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

#[derive(Clone)]
struct AppState {
    session: Arc<Mutex<RepositorySession>>,
    query_cache: Arc<RwLock<Option<Arc<QueryEngine>>>>,
}

#[derive(Debug, Deserialize)]
struct OpenRequest {
    repo_path: PathBuf,
    store_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    paths: Option<Vec<PathBuf>>,
}

#[derive(Debug, Deserialize)]
struct FindSymbolQuery {
    name: Option<String>,
    fq_name: Option<String>,
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DependencyQuery {
    target: String,
    direction: Option<String>,
    depth: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct TargetQuery {
    symbol: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = match args.store_path {
        Some(path) => RepositorySessionConfig::new(args.repo).with_store_path(path),
        None => RepositorySessionConfig::new(args.repo),
    };
    let session = RepositorySession::open(config)?;
    let indexer = Indexer::new(session.clone());
    if indexer.health()?.indexed_files == 0 {
        indexer.build_full()?;
    }
    let query_cache = Arc::new(RwLock::new(Some(Arc::new(session.query_engine()?))));

    let app = Router::new()
        .route("/index/open", post(open_index))
        .route("/index/refresh", post(refresh_index))
        .route("/graph/find_symbol", get(find_symbol))
        .route("/graph/dependencies", get(dependencies))
        .route("/graph/callers", get(callers))
        .route("/graph/callees", get(callees))
        .route("/graph/references", get(references))
        .route("/graph/impact", get(impact))
        .route("/graph/explain", get(explain))
        .with_state(AppState {
            session: Arc::new(Mutex::new(session)),
            query_cache,
        });

    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_index(State(state): State<AppState>, Json(request): Json<OpenRequest>) -> Response {
    let config = match request.store_path {
        Some(path) => RepositorySessionConfig::new(request.repo_path).with_store_path(path),
        None => RepositorySessionConfig::new(request.repo_path),
    };
    match RepositorySession::open(config) {
        Ok(session) => {
            let indexer = Indexer::new(session.clone());
            let result = indexer.build_full().and_then(|stats| {
                let mut guard = state.session.lock().expect("session mutex poisoned");
                *guard = session.clone();
                refresh_query_cache(&state, &session)?;
                Ok(stats)
            });
            response(result)
        }
        Err(error) => error_response(error),
    }
}

async fn refresh_index(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> impl IntoResponse {
    let session = clone_session(&state);
    let indexer = Indexer::new(session.clone());
    let paths = request
        .paths
        .unwrap_or_else(|| vec![indexer.session().repo_path().to_path_buf()]);
    let result = indexer.refresh_paths(&paths).and_then(|stats| {
        refresh_query_cache(&state, &session)?;
        Ok(stats)
    });
    response(result)
}

async fn find_symbol(
    State(state): State<AppState>,
    Query(query): Query<FindSymbolQuery>,
) -> impl IntoResponse {
    let kind = query.kind.as_deref().and_then(SymbolKind::parse);
    response(cached_query_engine(&state).and_then(|engine| {
        engine.find_symbol(QueryFilter {
            name: query.name,
            fq_name: query.fq_name,
            kind,
            name_contains: None,
            name_prefix: None,
            limit: Some(50),
        })
    }))
}

async fn dependencies(
    State(state): State<AppState>,
    Query(query): Query<DependencyQuery>,
) -> impl IntoResponse {
    let direction = match query.direction.as_deref() {
        Some("inbound") => DependencyDirection::Inbound,
        Some("both") => DependencyDirection::Both,
        _ => DependencyDirection::Outbound,
    };
    response(
        cached_query_engine(&state)
            .and_then(|engine| engine.dependencies(&query.target, direction, query.depth.unwrap_or(2))),
    )
}

async fn callers(
    State(state): State<AppState>,
    Query(query): Query<TargetQuery>,
) -> impl IntoResponse {
    let target = query.symbol.or(query.path).unwrap_or_default();
    response(
        cached_query_engine(&state).and_then(|engine| engine.callers(&target)),
    )
}

async fn callees(
    State(state): State<AppState>,
    Query(query): Query<TargetQuery>,
) -> impl IntoResponse {
    let target = query.symbol.or(query.path).unwrap_or_default();
    response(
        cached_query_engine(&state).and_then(|engine| engine.callees(&target)),
    )
}

async fn references(
    State(state): State<AppState>,
    Query(query): Query<TargetQuery>,
) -> impl IntoResponse {
    let target = query.symbol.or(query.path).unwrap_or_default();
    response(
        cached_query_engine(&state).and_then(|engine| engine.references(&target)),
    )
}

async fn impact(
    State(state): State<AppState>,
    Query(query): Query<DependencyQuery>,
) -> impl IntoResponse {
    response(
        cached_query_engine(&state)
            .and_then(|engine| engine.impact(&query.target, query.depth.unwrap_or(3))),
    )
}

async fn explain(
    State(state): State<AppState>,
    Query(query): Query<TargetQuery>,
) -> impl IntoResponse {
    let target = query.symbol.or(query.path).unwrap_or_default();
    response(
        cached_query_engine(&state).and_then(|engine| engine.explain(&target)),
    )
}

fn clone_session(state: &AppState) -> RepositorySession {
    state
        .session
        .lock()
        .expect("session mutex poisoned")
        .clone()
}

fn cached_query_engine(
    state: &AppState,
) -> Result<Arc<QueryEngine>, cortex_core::storage::CortexError> {
    {
        let guard = state.query_cache.read().expect("query cache lock poisoned");
        if let Some(engine) = guard.as_ref() {
            return Ok(Arc::clone(engine));
        }
    }
    let session = clone_session(state);
    let engine = Arc::new(session.query_engine()?);
    let mut guard = state.query_cache.write().expect("query cache lock poisoned");
    *guard = Some(Arc::clone(&engine));
    Ok(engine)
}

fn refresh_query_cache(
    state: &AppState,
    session: &RepositorySession,
) -> Result<(), cortex_core::storage::CortexError> {
    let engine = Arc::new(session.query_engine()?);
    let mut guard = state.query_cache.write().expect("query cache lock poisoned");
    *guard = Some(engine);
    Ok(())
}

fn response<T>(value: Result<T, cortex_core::storage::CortexError>) -> Response
where
    T: Serialize,
{
    match value {
        Ok(value) => Json(value).into_response(),
        Err(error) => error_response(error),
    }
}

fn error_response(error: impl std::fmt::Display) -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(ErrorBody {
            error: error.to_string(),
        }),
    )
        .into_response()
}
