use std::collections::HashSet;

use f6_types::LegalEntityTIN;
use f6_types::fns::EgrResponseItem;
use f6_types::report::TINReport;
use hickory_resolver::Resolver;
use hickory_resolver::config::{GOOGLE, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;

#[tracing::instrument(skip(egr))]
pub async fn build(tin: LegalEntityTIN, egr: EgrResponseItem) -> TINReport {
    let legal_entity = egr.legal_entity;

    let resolver = Resolver::builder_with_config(
        ResolverConfig::udp_and_tcp(&GOOGLE),
        TokioRuntimeProvider::default(),
    )
    .build()
    .unwrap();

    let mut ip_addrs = HashSet::new();
    for domain in &legal_entity.contacts.sites {
        if let Ok(lookup) = resolver.lookup_ip(domain.trim_matches('/')).await {
            for ip_addr in lookup.iter() {
                ip_addrs.insert(ip_addr);
            }
        }
    }

    TINReport {
        tin,
        name: legal_entity.short_name,
        domains: legal_entity.contacts.sites.into_iter().collect(),
        ip_addrs,
    }
}
