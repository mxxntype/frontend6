use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

use derive_more::{Deref, DerefMut, From};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

pub mod fns;

#[derive(Serialize, Deserialize, Hash, From, PartialEq, Eq, Clone, Debug)]
#[serde(untagged)]
pub enum ApiKey {
    Fns(fns::FnsApiKey),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    Usable,
    Exhausted,
}

#[derive(Serialize, Deserialize, Deref, DerefMut, Debug)]
pub struct ApiKeyRegistry(HashMap<ApiKey, KeyStatus>);

#[derive(Debug)]
pub struct ApiKeyRegistryStorage {
    pub file_path: PathBuf,
    pub registry: ApiKeyRegistry,
}

impl ApiKeyRegistryStorage {
    pub fn create_temporary() -> Result<(Self, NamedTempFile), ApiKeyRegistryStorageError> {
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_file_path = temp_file.path().to_path_buf();

        let registry_storage = Self {
            file_path: temp_file_path,
            registry: ApiKeyRegistry(HashMap::new()),
        };

        Ok((registry_storage, temp_file))
    }

    pub fn write(&self) -> Result<(), ApiKeyRegistryStorageError> {
        let registry_json = serde_json::to_string_pretty(&self.registry)?;
        fs::write(&self.file_path, registry_json).map_err(Into::into)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ApiKeyRegistryStorageError {
    #[error("An I/O error occurred")]
    Io(#[from] io::Error),
    #[error("`serde_json` returned an error")]
    SerdeJson(#[from] serde_json::Error),
}
