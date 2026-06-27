use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SUBDIR_EGR: &str = "egr";
pub const SUBDIR_DOMAIN: &str = "domain";
pub const SUBDIR_IP_ADDR: &str = "ip";
pub const SUBDIR_INFRA: &str = "infra";

#[derive(Debug)]
#[must_use]
pub struct Cache<I, T> {
    path: PathBuf,
    marker_i: std::marker::PhantomData<I>,
    marker_t: std::marker::PhantomData<T>,
}

impl<I, T> Cache<I, T>
where
    I: std::fmt::Display + Send + Sync,
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let path = std::env::current_dir()?.join("cache").join(path);
        std::fs::create_dir_all(&path)?;
        tracing::debug!(
            "Creating Cache<{i}, {t}> in {path}",
            i = std::any::type_name::<I>(),
            t = std::any::type_name::<T>(),
            path = path.display(),
        );
        Ok(Self {
            path,
            marker_i: std::marker::PhantomData,
            marker_t: std::marker::PhantomData,
        })
    }

    pub fn file_path(&self, id: &I) -> PathBuf {
        self.path.join(format!("{id}.json"))
    }

    #[tracing::instrument(skip_all, fields(%id), err(Debug))]
    pub async fn persist(&self, id: &I, value: &T) -> Result<PathBuf, CacheError> {
        let path = self.file_path(id);
        let json = serde_json::to_string_pretty(value).unwrap();
        tokio::fs::write(&path, &json)
            .await
            .inspect(|()| tracing::debug!("Persisted value in cache"))?;
        Ok(path)
    }

    #[tracing::instrument(skip_all, fields(%id), err(Debug))]
    pub async fn retrieve(&self, id: &I) -> Result<Option<T>, CacheError> {
        let path = self.file_path(id);
        if tokio::fs::try_exists(&path).await? {
            let json = tokio::fs::read_to_string(&path).await?;
            let value: T = serde_json::from_str(&json)
                .inspect(|_| tracing::debug!("Retrieved value from cache"))?;
            Ok(Some(value))
        } else {
            tracing::debug!("Value not found in cache");
            Ok(None)
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum CacheError {
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
}
