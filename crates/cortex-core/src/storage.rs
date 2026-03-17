use crate::model::PersistedState;
use serde_json::Error as SerdeError;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
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
    #[error("indexing memory budget exceeded during {phase}: {usage_mb} MB used (limit {limit_mb} MB). Set CORTEX_MAX_INDEX_MEMORY_MB to adjust or 0 to disable.")]
    MemoryBudgetExceeded {
        phase: String,
        usage_mb: u64,
        limit_mb: u64,
    },
    #[error("invalid CORTEX_MAX_INDEX_MEMORY_MB value '{value}': expected a positive integer or 0 to disable the guard")]
    InvalidMemoryBudget { value: String },
    #[error("unable to read current process memory usage: {0}")]
    MemoryUsage(String),
}

#[derive(Debug)]
pub struct SledGraphStore {
    root: PathBuf,
}

impl SledGraphStore {
    pub fn open(root: &Path) -> Result<Self, CortexError> {
        let normalized = Self::normalize_store_path(root);
        std::fs::create_dir_all(&normalized)?;
        let store = Self { root: normalized };
        store.recover_temp_stores()?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_state(&self) -> Result<PersistedState, CortexError> {
        let path = snapshot_path(&self.root);
        let snapshot_error = if path.exists() {
            match read_snapshot(&path) {
                Ok(state) => return Ok(state),
                Err(error) => Some(error),
            }
        } else {
            None
        };
        let db = self.open_db()?;
        let Some(raw) = db.get(STATE_KEY)? else {
            if let Some(error) = snapshot_error {
                return Err(error);
            }
            return Ok(PersistedState::default());
        };
        Ok(serde_json::from_slice(raw.as_ref())?)
    }

    pub fn save_state(&self, state: &PersistedState) -> Result<(), CortexError> {
        Self::write_state_to(&self.root, state)?;
        Self::write_snapshot(&self.root, state)?;
        Ok(())
    }

    /// Replace the on-disk store with a freshly written copy.
    /// Ensures only the latest store remains on disk.
    pub fn replace_store(&self, state: &PersistedState) -> Result<u64, CortexError> {
        let (next, prev) = self.store_swap_paths();

        Self::remove_path(&next)?;
        Self::remove_path(&prev)?;

        std::fs::create_dir_all(&next)?;
        Self::write_state_to(&next, state)?;
        let size_before = Self::dir_size(&self.root).unwrap_or(0);

        let had_root = self.root.exists();
        if had_root {
            std::fs::rename(&self.root, &prev)?;
        }

        if let Err(error) = std::fs::rename(&next, &self.root) {
            if had_root {
                let _ = std::fs::rename(&prev, &self.root);
            }
            return Err(error.into());
        }

        Self::write_snapshot(&self.root, state)?;
        Self::remove_path(&prev)?;

        let size_after = Self::dir_size(&self.root).unwrap_or(0);
        Ok(size_before.saturating_sub(size_after))
    }

    fn open_db(&self) -> Result<sled::Db, CortexError> {
        Ok(sled::open(&self.root)?)
    }

    fn write_state_to(root: &Path, state: &PersistedState) -> Result<(), CortexError> {
        let db = sled::open(root)?;
        let bytes = serde_json::to_vec(state)?;
        db.insert(STATE_KEY, bytes)?;
        db.flush()?;
        Ok(())
    }

    fn write_snapshot(root: &Path, state: &PersistedState) -> Result<(), CortexError> {
        let snapshot_path = snapshot_path(root);
        let file = File::create(snapshot_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, state)?;
        writer.flush()?;
        Ok(())
    }

    fn recover_temp_stores(&self) -> Result<(), CortexError> {
        let (next, prev) = self.store_swap_paths();

        if self.root.exists() {
            Self::remove_path(&next)?;
            Self::remove_path(&prev)?;
            return Ok(());
        }

        if next.exists() {
            std::fs::rename(&next, &self.root)?;
            Self::remove_path(&prev)?;
            return Ok(());
        }

        if prev.exists() {
            std::fs::rename(&prev, &self.root)?;
        }

        Ok(())
    }

    fn store_swap_paths(&self) -> (PathBuf, PathBuf) {
        let parent = self.root.parent().unwrap_or(self.root.as_path());
        (parent.join("index.next"), parent.join("index.prev"))
    }

    fn remove_path(path: &Path) -> Result<(), CortexError> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn normalize_store_path(path: &Path) -> PathBuf {
        let mut normalized = path.to_path_buf();
        if normalized
            .file_name()
            .is_some_and(|name| name.to_string_lossy() == "index")
        {
            return normalized;
        }
        if normalized.extension().is_some() {
            normalized.set_extension("");
        }
        if normalized
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains("index"))
        {
            return normalized;
        }
        normalized.push("index");
        normalized
    }

    fn dir_size(path: &Path) -> Result<u64, CortexError> {
        if !path.exists() {
            return Ok(0);
        }
        let mut size = 0u64;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(std::io::Error::other)?;
            if entry.file_type().is_file() {
                size += entry
                    .metadata()
                    .map_err(std::io::Error::other)?
                    .len();
            }
        }
        Ok(size)
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
        let state = read_snapshot(&path)?;
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

fn read_snapshot(path: &Path) -> Result<PersistedState, CortexError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
