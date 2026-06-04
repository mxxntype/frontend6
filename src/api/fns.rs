use super::Api;
use crate::key_registry::fns::FnsApiKey;
use crate::types::LegalEntityTIN;

#[derive(Debug)]
#[must_use]
pub struct FnsApi;

impl FnsApi {
    pub const FNS_API_EGR_URL: &str = "https://api-fns.ru/api/egr";
}

impl Api for FnsApi {
    type ApiKeyType = FnsApiKey;

    async fn fetch_egr(
        &self,
        api_key: Self::ApiKeyType,
        tin: LegalEntityTIN,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let url = format!(
            "{base_uri}?key={key}&req={tin}",
            base_uri = Self::FNS_API_EGR_URL,
            key = api_key,
        );

        reqwest::get(url).await?.json().await
    }
}
