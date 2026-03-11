use crate::model::PersistedState;
use serde_json::Error as SerdeError;
use std::path::{Path, PathBuf};
use thiserror::Error;

const STATE_KEY: &[u8] = b"state";

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
    db: sled::Db,
}

impl SledGraphStore {
    pub fn open(root: &Path) -> Result<Self, CortexError> {
        let db = sled::open(root)?;
        Ok(Self { db })
    }

    pub fn load_state(&self) -> Result<PersistedState, CortexError> {
        let Some(raw) = self.db.get(STATE_KEY)? else {
            return Ok(PersistedState::default());
        };
        Ok(serde_json::from_slice(raw.as_ref())?)
    }

    pub fn save_state(&self, state: &PersistedState) -> Result<(), CortexError> {
        let bytes = serde_json::to_vec(state)?;
        self.db.insert(STATE_KEY, bytes)?;
        self.db.flush()?;
        Ok(())
    }
}
