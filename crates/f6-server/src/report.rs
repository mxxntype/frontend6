use std::collections::HashSet;

use f6_types::LegalEntityTIN;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponseItem;
use f6_types::ip_addr::IpAddrResponse;
use f6_types::report::TINReport;
use itertools::Itertools;
use tempfile::NamedTempFile;

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

    let report = TINReport {
        tin,
        name: legal_entity.short_name,
        domains,
        ip_addrs,
    };

    self::build_pdf(&report);

    report
}

#[tracing::instrument(skip_all, fields(%tin))]
pub fn build_pdf(
    TINReport {
        tin,
        name,
        domains,
        ip_addrs,
    }: &TINReport,
) {
    let typst_source = format!(
        "
= Отчёт о сканировании ИНН {tin}

Организация: *{name}*

== Обнаруженные домены и поддомены:
{domains}

== IP-адреса:

{ips}
",
        domains = domains
            .iter()
            .map(|domain| format!("\n - `{domain}`"))
            .join(""),
        ips = ip_addrs.iter().map(|ip| format!("\n - `{ip}`")).join(""),
    );

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
