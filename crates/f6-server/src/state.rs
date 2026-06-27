use std::collections::HashMap;
use std::sync::Arc;

use f6_types::LegalEntityTIN;
use f6_types::as_info::ASInfo;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponse;
use f6_types::ip_addr::IpAddrResponse;
use tokio::sync::Mutex as AsyncMutex;

use crate::cache::Cache;

#[derive(Clone)]
#[must_use]
pub struct ServerState {
    pub fns_api_key: String,
    pub cache_egr: Arc<AsyncMutex<Cache<LegalEntityTIN, EgrResponse>>>,
    pub cache_domain: Arc<AsyncMutex<Cache<LegalEntityTIN, DomainResponse>>>,
    pub cache_ip: Arc<AsyncMutex<Cache<LegalEntityTIN, IpAddrResponse>>>,
    pub cache_as_info: Arc<AsyncMutex<Cache<LegalEntityTIN, HashMap<u64, ASInfo>>>>,
}
