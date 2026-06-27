use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use f6_types::LegalEntityTIN;
use f6_types::as_info::ASInfo;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponseItem;
use f6_types::ip_addr::IpAddrResponse;
use f6_types::report::TINReport;
use tempfile::NamedTempFile;

#[tracing::instrument(skip_all, fields(%tin))]
#[expect(clippy::implicit_hasher, reason = "who gives a shit")]
pub async fn build(
    tin: LegalEntityTIN,
    egr_response: EgrResponseItem,
    DomainResponse(sublist3r_domains): DomainResponse,
    IpAddrResponse(ip_addrs): IpAddrResponse,
    ripe_info: HashMap<u64, ASInfo>,
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

    let report = TINReport {
        tin,
        name: legal_entity.short_name,
        domains,
        ip_addrs,
        ripe_info,
    };

    self::build_pdf(&report);

    report
}

#[tracing::instrument(skip_all, fields(%tin))]
pub fn build_pdf(
    TINReport {
        tin,
        name,
        domains: _,
        ip_addrs,
        ripe_info,
    }: &TINReport,
) {
    let mut typst_source = format!(
        "
= Отчёт о сканировании ИНН {tin}

Организация: *{name}*

= Обнаруженные домены и поддомены:

",
    );

    for (domain, ip) in ip_addrs {
        writeln!(typst_source, "- `{domain} ({ip})`").unwrap();
    }

    for (asn, as_info) in ripe_info {
        let holder = as_info.holder.as_ref().map_or("неизвестен", |v| v.as_str());
        writeln!(typst_source, "= AS {asn} (владелец: {holder}) ").unwrap();
        for (domain, ip) in &as_info.domains {
            writeln!(typst_source, "- `{domain} ({ip})`").unwrap();
        }
    }

    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), typst_source).unwrap();

    let command_ok = std::process::Command::new("typst")
        .arg("compile")
        .arg(temp_file.path())
        .arg(format!("cache/report/{tin}.pdf"))
        .status()
        .unwrap()
        .success();
    assert!(command_ok, "typst failed!");
}
