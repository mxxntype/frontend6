use std::sync::Arc;

use f6_types::{LegalEntityTIN, fns::EgrResponse};
use tokio::sync::Mutex as AsyncMutex;

use crate::cache::Cache;

#[derive(Clone)]
#[must_use]
pub struct ServerState {
    pub fns_api_key: String,
    pub egr_cache: Arc<AsyncMutex<Cache<LegalEntityTIN, EgrResponse>>>,
}
