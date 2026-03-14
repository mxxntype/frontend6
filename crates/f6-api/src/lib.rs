use f6_types::LegalEntityTIN;

pub mod fns;

pub trait Api {
    type ApiKeyType;

    fn fetch_egr(&self, api_key: Self::ApiKeyType, tin: &LegalEntityTIN) -> String;
}
