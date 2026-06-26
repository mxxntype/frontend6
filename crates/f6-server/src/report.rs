use std::collections::HashSet;

use f6_types::LegalEntityTIN;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponseItem;
use f6_types::ip_addr::IpAddrResponse;
use f6_types::report::TINReport;

#[tracing::instrument(skip_all, fields(%tin))]
pub async fn build(
    tin: LegalEntityTIN,
    egr_response: EgrResponseItem,
    DomainResponse(sublist3r_domains): DomainResponse,
    IpAddrResponse(ip_addrs): IpAddrResponse,
) -> TINReport {
    let legal_entity = egr_response.legal_entity;

    let known_domains = legal_entity
        .contacts
        .domains
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let domains = sublist3r_domains
        .union(&known_domains)
        .cloned()
        .collect::<HashSet<_>>();

    TINReport {
        tin,
        name: legal_entity.short_name,
        domains,
        ip_addrs,
    }
}
