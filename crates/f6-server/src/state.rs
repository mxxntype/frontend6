use std::sync::Arc;

use f6_types::{
    LegalEntityTIN, domain::DomainResponse, fns::EgrResponse, ip_addr::IpAddrResponse,
    report::InfrastructureGroup,
};
use tokio::sync::Mutex as AsyncMutex;

use crate::cache::Cache;

#[derive(Clone)]
#[must_use]
pub struct ServerState {
    pub fns_api_key: String,
    pub cache_egr: Arc<AsyncMutex<Cache<LegalEntityTIN, EgrResponse>>>,
    pub cache_domain: Arc<AsyncMutex<Cache<LegalEntityTIN, DomainResponse>>>,
    pub cache_ip: Arc<AsyncMutex<Cache<LegalEntityTIN, IpAddrResponse>>>,
    pub cache_infra: Arc<AsyncMutex<Cache<LegalEntityTIN, Vec<InfrastructureGroup>>>>,
}
