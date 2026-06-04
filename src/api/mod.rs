use crate::types::LegalEntityTIN;

pub mod fns;

pub(crate) trait Api {
    type ApiKeyType;

    async fn fetch_egr(
        &self,
        api_key: Self::ApiKeyType,
        tin: LegalEntityTIN,
    ) -> Result<serde_json::Value, reqwest::Error>;
}
