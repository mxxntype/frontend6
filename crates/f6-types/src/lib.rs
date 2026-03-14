use nutype::nutype;

#[nutype(
    validate(greater_or_equal = 1_000_000_000, less_or_equal = 9_999_999_999),
    derive(
        Serialize,
        Deserialize,
        Hash,
        Display,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        FromStr,
        Deref,
    )
)]
pub struct LegalEntityTIN(u64);

#[cfg(test)]
mod tests {
    use crate::LegalEntityTIN;

    #[rstest::rstest]
    #[case::valid_redsoft(9_705_000_373, true)]
    #[case::valid_ozon(7_704_217_370, true)]
    fn legal_entity_tin_validation(#[case] tin_numeric: u64, #[case] valid: bool) {
        let result = LegalEntityTIN::try_new(tin_numeric);
        match valid {
            true => assert!(result.is_ok_and(|tin| tin.into_inner() == tin_numeric)),
            false => assert!(result.is_err()),
        }
    }
}
