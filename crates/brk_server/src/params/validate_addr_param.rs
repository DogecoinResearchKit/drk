use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ValidateAddrParam {
    /// Address to validate (can be any string)
    #[cfg_attr(
        all(feature = "dogecoin", not(feature = "litecoin")),
        schemars(
            example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
            example = &"DPLUJYD5fLTybiPWBH2hKqW7Ek9cbir1Mm",
            example = &"not-a-valid-addr",
            example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
        )
    )]
    #[cfg_attr(
        feature = "litecoin",
        schemars(
            example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
            example = &"LdRL2VaGSaokKWu4cq2S46QGZpnbRYJW56",
            example = &"not-a-valid-addr",
            example = &"ltc1qinvalid"
        )
    )]
    #[cfg_attr(
        not(any(feature = "litecoin", feature = "dogecoin")),
        schemars(
            example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
            example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
            example = &"not-a-valid-addr",
            example = &"bc1qinvalid"
        )
    )]
    #[serde(rename = "address")]
    pub addr: String,
}
