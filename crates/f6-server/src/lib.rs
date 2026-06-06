use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use f6_types::LegalEntityTIN;
use f6_types::fns::EgrResponse;
use f6_types::report::TINReport;
use reqwest::StatusCode;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::cache::Cache;
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
    let egr_cache = Arc::new(Mutex::new(Cache::new(&PathBuf::from(cache::SUBDIR_EGR))?));
    let state = ServerState { egr_cache };
    let router = Router::new()
        .route("/egr/{tin}", get(read_egr))
        .route("/report/{tin}", get(build_report))
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(state);

    Ok(router)
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn read_egr(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<EgrResponse>, StatusCode> {
    if let Some(egr) = state
        .egr_cache
        .lock()
        .await
        .retrieve(&tin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(egr));
    }

    // TODO: Query FNS-API for EGR of this TIN.
    tracing::warn!("EGR for this TIN has not been cached, cannot return anything");

    Err(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn build_report(
    State(state): State<ServerState>,
    Path(tin): Path<LegalEntityTIN>,
) -> Result<Json<TINReport>, StatusCode> {
    if let Some(egr) = state
        .egr_cache
        .lock()
        .await
        .retrieve(&tin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let report = self::report::build(tin, egr.items[0].clone()).await;
        return Ok(Json(report));
    }

    // TODO: Query FNS-API for EGR of this TIN.
    tracing::warn!("EGR for this TIN has not been cached, cannot return anything");

    Err(StatusCode::NO_CONTENT)
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("An I/O error occurred")]
    Io(#[from] std::io::Error),
}
