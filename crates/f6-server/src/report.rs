use std::collections::HashSet;

use f6_types::LegalEntityTIN;
use f6_types::fns::EgrResponseItem;
use f6_types::report::TINReport;
use hickory_resolver::Resolver;
use hickory_resolver::config::{GOOGLE, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use itertools::Itertools;
use tempfile::NamedTempFile;

#[tracing::instrument(skip(egr))]
pub async fn build(tin: LegalEntityTIN, egr: EgrResponseItem) -> TINReport {
    let legal_entity = egr.legal_entity;

    let shortest_domain = legal_entity
        .contacts
        .domains
        .iter()
        .sorted_unstable_by_key(|d| d.len())
        .next()
        .unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    let command_ok = std::process::Command::new("python3")
        .arg("libs/sublist3r/sublist3r.py")
        .arg("-d")
        .arg(shortest_domain)
        .arg("-o")
        .arg(temp_file.path())
        .status()
        .unwrap()
        .success();
    assert!(command_ok, "sublist3r failed!");

    let known_domains = legal_entity
        .contacts
        .domains
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let sublist3r_domains = std::fs::read_to_string(temp_file.path())
        .unwrap()
        .lines()
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    let domains = sublist3r_domains
        .union(&known_domains)
        .cloned()
        .collect::<HashSet<_>>();

    let resolver = Resolver::builder_with_config(
        ResolverConfig::udp_and_tcp(&GOOGLE),
        TokioRuntimeProvider::default(),
    )
    .build()
    .unwrap();

    let mut ip_addrs = HashSet::new();
    for domain in &domains {
        if let Ok(lookup) = resolver.lookup_ip(domain.trim_matches('/')).await {
            for ip_addr in lookup.iter() {
                ip_addrs.insert(ip_addr);
            }
        }
    }

    TINReport {
        tin,
        name: legal_entity.short_name,
        domains,
        ip_addrs,
    }
}
