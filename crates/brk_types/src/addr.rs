use brk_chain::primitives as bitcoin;
use std::{fmt, str::FromStr};

use bitcoin::ScriptBuf;
use brk_chain::address::{Address, MAINNET};
use brk_error::Error;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::Formattable;

use crate::AddrBytes;

use super::OutputType;

/// Address string for the active chain
///
/// Examples are chain-specific: an API served from a Dogecoin build must not
/// advertise `1...`/`bc1...` addresses, which no Dogecoin wallet can spend.
#[derive(Debug, Clone, Deref, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[cfg_attr(
    all(feature = "dogecoin", not(feature = "litecoin")),
    schemars(
        example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        example = &"DPLUJYD5fLTybiPWBH2hKqW7Ek9cbir1Mm",
        example = &"AAdeRfpmytky4FGoyvN9SqKpCi64wVYuNA"
    )
)]
#[cfg_attr(
    feature = "litecoin",
    schemars(
        example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        example = &"LdRL2VaGSaokKWu4cq2S46QGZpnbRYJW56",
        example = &"MS6XziAqrwjVxPBEffh52LwqpqJUmMtpvn"
    )
)]
#[cfg_attr(
    not(any(feature = "litecoin", feature = "dogecoin")),
    schemars(
        example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        example = &"bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"
    )
)]
pub struct Addr(String);

impl Addr {
    /// Get the script for this address
    pub fn script(&self) -> Result<ScriptBuf, Error> {
        AddrBytes::addr_to_script(&self.0)
    }
}

impl From<String> for Addr {
    #[inline]
    fn from(addr: String) -> Self {
        Self(addr)
    }
}

impl TryFrom<&ScriptBuf> for Addr {
    type Error = Error;
    fn try_from(script: &ScriptBuf) -> Result<Self, Self::Error> {
        Self::try_from((script, OutputType::from(script)))
    }
}

impl TryFrom<(&ScriptBuf, OutputType)> for Addr {
    type Error = Error;
    fn try_from((script, output_type): (&ScriptBuf, OutputType)) -> Result<Self, Self::Error> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 => {
                // P2PK has no standard address encoding, use raw pubkey hex
                let bytes = AddrBytes::try_from((script, output_type))?;
                Ok(Self(bytes_to_hex(bytes.as_slice())))
            }
            _ if output_type.is_addr() => {
                // `MAINNET` is the active chain's mainnet, not necessarily
                // Bitcoin's. On Dogecoin this rejects witness programs, which
                // have no address encoding there — hence the fallible path.
                let addr = Address::from_script(script, MAINNET).map_err(|_| Error::InvalidAddr)?;
                Ok(Self(addr.to_string()))
            }
            _ => Err(Error::InvalidAddr),
        }
    }
}

impl TryFrom<&AddrBytes> for Addr {
    type Error = Error;
    fn try_from(bytes: &AddrBytes) -> Result<Self, Self::Error> {
        Self::try_from(&bytes.to_script_pubkey())
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut hex_string, "{:02x}", byte).unwrap();
    }
    hex_string
}

impl FromStr for Addr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = AddrBytes::addr_to_script(s)?;
        Ok(Self(s.to_string()))
    }
}

impl Formattable for Addr {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.0.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// hash160 of a P2PKH output in block
    /// `1fe8174c6f2bb43fe746033995e4757ae5c33e2c46106ce846a97a19384e8dfc`.
    const HASH160: &str = "c79901c495b8cf5416a4fb0bcb367f179ae2d910";

    /// `HASH160` encoded for the chain this build targets.
    ///
    /// `brk_chain::primitives` swaps the primitives *crate* per feature, but a
    /// `Network` **variant** named at a call site does not follow it: on
    /// `bitcoin-dogecoin` the root namespace is still Bitcoin, so
    /// `Network::Bitcoin` silently encodes version byte `0x00`. Nothing about
    /// that failure is visible to the compiler, so pin the bytes instead.
    #[cfg(all(feature = "dogecoin", not(feature = "litecoin")))]
    const P2PKH: &str = "DPLUJYD5fLTybiPWBH2hKqW7Ek9cbir1Mm";
    #[cfg(all(feature = "dogecoin", not(feature = "litecoin")))]
    const P2SH: &str = "AAdeRfpmytky4FGoyvN9SqKpCi64wVYuNA";

    #[cfg(feature = "litecoin")]
    const P2PKH: &str = "LdRL2VaGSaokKWu4cq2S46QGZpnbRYJW56";
    #[cfg(feature = "litecoin")]
    const P2SH: &str = "MS6XziAqrwjVxPBEffh52LwqpqJUmMtpvn";

    #[cfg(not(any(feature = "litecoin", feature = "dogecoin")))]
    const P2PKH: &str = "1KCNmHGSMvZh4iCuSh38n5LWMcRKCeEbpM";
    #[cfg(not(any(feature = "litecoin", feature = "dogecoin")))]
    const P2SH: &str = "3KtPgpksupt59suLZnhjChhSW8i2mBqi62";

    #[test]
    fn p2pkh_round_trips_through_the_active_chains_encoding() {
        let script = AddrBytes::addr_to_script(P2PKH).expect("address parses");
        assert_eq!(bytes_to_hex(&script.as_bytes()[3..23]), HASH160);
        assert_eq!(
            Addr::try_from(&script).expect("script encodes").as_str(),
            P2PKH
        );
    }

    #[test]
    fn p2sh_round_trips_through_the_active_chains_encoding() {
        let script = AddrBytes::addr_to_script(P2SH).expect("address parses");
        assert_eq!(bytes_to_hex(&script.as_bytes()[2..22]), HASH160);
        assert_eq!(
            Addr::try_from(&script).expect("script encodes").as_str(),
            P2SH
        );
    }

    /// A `1...` string is a *valid Bitcoin address*, so rendering one on an
    /// altcoin build hands users a payout target no one on that chain can
    /// claim. Refusing to resolve one is the other half of that guarantee.
    #[cfg(any(feature = "litecoin", feature = "dogecoin"))]
    #[test]
    fn bitcoin_mainnet_p2pkh_does_not_resolve() {
        assert!(AddrBytes::addr_to_script("1KCNmHGSMvZh4iCuSh38n5LWMcRKCeEbpM").is_err());
    }

    /// Litecoin is deliberately excluded: Litecoin Core keeps `0x05` as a
    /// legacy mainnet P2SH prefix (`chainparams.cpp` `SCRIPT_ADDRESS2`), so a
    /// `3...` address really is spendable there and must keep decoding.
    /// Dogecoin has no such alias — `0x16` is the only mainnet P2SH prefix.
    #[cfg(all(feature = "dogecoin", not(feature = "litecoin")))]
    #[test]
    fn bitcoin_mainnet_p2sh_does_not_resolve() {
        assert!(AddrBytes::addr_to_script("3KtPgpksupt59suLZnhjChhSW8i2mBqi62").is_err());
    }
}
