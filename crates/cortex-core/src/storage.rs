use crate::model::PersistedState;
use serde_json::Error as SerdeError;
use std::path::{Path, PathBuf};
use thiserror::Error;

const STATE_KEY: &[u8] = b"state";
const STATE_SNAPSHOT: &str = "state.json";

#[derive(Debug, Error)]
pub enum CortexError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage error: {0}")]
    Storage(#[from] sled::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerdeError),
    #[error("unsupported language for path {0}")]
    UnsupportedLanguage(PathBuf),
    #[error("query target not found: {0}")]
    NotFound(String),
    #[error("parser error: {0}")]
    Parser(String),
}

#[derive(Debug)]
pub struct SledGraphStore {
    root: PathBuf,
    db: sled::Db,
}

impl SledGraphStore {
    pub fn open(root: &Path) -> Result<Self, CortexError> {
        let db = sled::open(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            db,
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_state(&self) -> Result<PersistedState, CortexError> {
        let Some(raw) = self.db.get(STATE_KEY)? else {
            return Ok(PersistedState::default());
        };
        Ok(serde_json::from_slice(raw.as_ref())?)
    }

    pub fn save_state(&self, state: &PersistedState) -> Result<(), CortexError> {
        let bytes = serde_json::to_vec(state)?;
        self.db.insert(STATE_KEY, bytes.clone())?;
        self.db.flush()?;
        // Write a side-channel JSON snapshot so read-only consumers can
        // load state without acquiring the exclusive sled lock.
        let snapshot_path = snapshot_path(&self.root);
        std::fs::write(snapshot_path, bytes)?;
        Ok(())
    }
    /// Compact the database by reloading and rewriting state.
    /// This reclaims space from accumulated blob segments.
    pub fn compact(&self) -> Result<u64, CortexError> {
        let size_before = self.db.size_on_disk()?;
        let state = self.load_state()?;
        self.db.flush()?;
        self.save_state(&state)?;
        let size_after = self.db.size_on_disk()?;
        Ok(size_before.saturating_sub(size_after))
    }
}

/// A read-only view of the persisted graph state loaded from the JSON
/// snapshot written alongside the sled store. Multiple processes can
/// read this file concurrently without blocking the writer.
pub struct ReadOnlyStore {
    state: PersistedState,
}

impl ReadOnlyStore {
    /// Attempt to load the JSON snapshot from `store_path`. Returns
    /// `None` when no snapshot exists yet (index has never been built).
    pub fn load(store_path: &Path) -> Result<Option<Self>, CortexError> {
        let path = snapshot_path(store_path);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)?;
        let state: PersistedState = serde_json::from_slice(&bytes)?;
        Ok(Some(Self { state }))
    }

    pub fn state(&self) -> &PersistedState {
        &self.state
    }

    pub fn into_state(self) -> PersistedState {
        self.state
    }
}

fn snapshot_path(store_path: &Path) -> PathBuf {
    // store_path is .cortex/index — put the snapshot one level up at
    // .cortex/state.json so it lives next to the sled directory.
    store_path
        .parent()
        .unwrap_or(store_path)
        .join(STATE_SNAPSHOT)
}
