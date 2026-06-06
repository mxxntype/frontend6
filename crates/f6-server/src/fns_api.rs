use f6_types::{LegalEntityTIN, fns::EgrResponse};

pub const FNS_API_EGR_URL: &str = "https://api-fns.ru/api/egr";

#[tracing::instrument(skip(api_key), err(Debug))]
pub async fn fetch_egr(api_key: &str, tin: LegalEntityTIN) -> Result<EgrResponse, reqwest::Error> {
    let url = format!("{FNS_API_EGR_URL}?key={api_key}&req={tin}");
    reqwest::get(url).await?.json().await
}
