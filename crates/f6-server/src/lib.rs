use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use f6_types::LegalEntityTIN;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponse;
use f6_types::ip_addr::IpAddrResponse;
use f6_types::report::TINReport;
use hickory_resolver::Resolver;
use hickory_resolver::config::{GOOGLE, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use itertools::Itertools;
use reqwest::StatusCode;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::cache::{Cache, SUBDIR_DOMAIN, SUBDIR_EGR, SUBDIR_IP_ADDR};
use crate::state::ServerState;

pub mod cache;
pub mod fns_api;
pub mod report;
pub mod state;

pub const DEFAULT_LISTEN_PORT: u16 = 8080;
pub const DEFAULT_LISTEN_ADDR: SocketAddrV4 =
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_LISTEN_PORT);

#[derive(Parser, Debug)]
pub struct Settings {
    #[arg(long("addr"), default_value_t = DEFAULT_LISTEN_ADDR)]
    pub addr: SocketAddrV4,
}

pub async fn run(settings: Settings) -> Result<(), ServerError> {
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let router = router()?;
    let listener = TcpListener::bind(settings.addr)
        .await
        .inspect(|_| tracing::debug!(addr = ?settings.addr, "Bound to TCP socket"))?;

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tracing::debug!("Stopping server");
        token.cancel();
    });

    tracing::debug!("Starting axum server");
    tokio::select! {
        axum_result = axum::serve(listener, router) => { let () = axum_result?; },
        () = cloned_token.cancelled() => {}
    }

    Ok(())
}

pub fn router() -> std::io::Result<Router> {
    const PATH: &str = "fns_api_key.txt";

    let Some(fns_api_key) = std::fs::read_to_string(PATH).ok() else {
        let key_path = std::env::current_dir().unwrap().join(PATH);
        tracing::error!("Please put your FNS-API key in {}", key_path.display());
        std::process::exit(1);
    };

    let cache_egr = Arc::new(Mutex::new(Cache::new(&PathBuf::from(SUBDIR_EGR))?));
    let cache_domain = Arc::new(Mutex::new(Cache::new(&PathBuf::from(SUBDIR_DOMAIN))?));
    let cache_ip = Arc::new(Mutex::new(Cache::new(&PathBuf::from(SUBDIR_IP_ADDR))?));

    let state = ServerState {
        fns_api_key: fns_api_key.trim().to_owned(),
        cache_egr,
        cache_domain,
        cache_ip,
    };

    let router = Router::new()
        .nest_service("/pdf", ServeDir::new("cache/report"))
        .route("/egr/{tin}", get(endpoint_egr))
        .route("/domain/{tin}", get(endpoint_domain))
        .route("/ip/{tin}", get(endpoint_ip))
        .route("/report/{tin}", get(endpoint_report))
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(state);

    Ok(router)
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn endpoint_egr(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<EgrResponse>, StatusCode> {
    if let Some(egr) = state
        .cache_egr
        .lock()
        .await
        .retrieve(&tin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(egr));
    }

    tracing::warn!("EGR for this TIN has not been cached, querying FNS-API");

    let egr = self::fns_api::fetch_egr(&state.fns_api_key, tin)
        .await
        .inspect_err(|error| tracing::error!(?error, "Failed to query FNS-API for EGR"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .cache_egr
        .lock()
        .await
        .persist(&tin, &egr)
        .await
        .unwrap();

    Ok(Json(egr))
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn endpoint_domain(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<DomainResponse>, StatusCode> {
    let Json(egr) = self::endpoint_egr(State(state.clone()), Path(tin))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(domain) = state
        .cache_domain
        .lock()
        .await
        .retrieve(&tin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(domain));
    }

    tracing::warn!("Domains for this TIN have not been cached, querying sublist3r");

    let tld = egr.items[0]
        .legal_entity
        .contacts
        .domains
        .iter()
        .sorted_unstable_by_key(|domain| domain.len())
        .next()
        .unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    let command_ok = std::process::Command::new("python3")
        .arg("libs/sublist3r/sublist3r.py")
        .arg("-d")
        .arg(tld)
        .arg("-o")
        .arg(temp_file.path())
        .status()
        .unwrap()
        .success();
    assert!(command_ok, "sublist3r failed!");

    let sublist3r_domains = std::fs::read_to_string(temp_file.path())
        .unwrap()
        .lines()
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    let domain = DomainResponse(sublist3r_domains);

    state
        .cache_domain
        .lock()
        .await
        .persist(&tin, &domain)
        .await
        .unwrap();

    Ok(Json(domain))
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn endpoint_ip(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<IpAddrResponse>, StatusCode> {
    if let Some(ip) = state
        .cache_ip
        .lock()
        .await
        .retrieve(&tin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(ip));
    }

    tracing::warn!("IPs for this TIN have not been cached, resolving");

    let Json(domain) = self::endpoint_domain(State(state.clone()), Path(tin))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resolver = Resolver::builder_with_config(
        ResolverConfig::udp_and_tcp(&GOOGLE),
        TokioRuntimeProvider::default(),
    )
    .build()
    .unwrap();

    let mut ip = HashSet::new();
    for domain in &domain.0 {
        if let Ok(lookup) = resolver.lookup_ip(domain.trim_matches('/')).await {
            for ip_addr in lookup.iter() {
                ip.insert(ip_addr);
            }
        }
    }

    let ip = IpAddrResponse(ip);

    state
        .cache_ip
        .lock()
        .await
        .persist(&tin, &ip)
        .await
        .unwrap();

    Ok(Json(ip))
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn endpoint_report(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<TINReport>, StatusCode> {
    let Json(egr) = self::endpoint_egr(State(state.clone()), Path(tin))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Json(domain_response) = self::endpoint_domain(State(state.clone()), Path(tin))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Json(ip_addr_response) = self::endpoint_ip(State(state), Path(tin))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let egr_response = egr.items[0].clone();
    let report = self::report::build(tin, egr_response, domain_response, ip_addr_response).await;

    Ok(Json(report))
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("An I/O error occurred")]
    Io(#[from] std::io::Error),
}
