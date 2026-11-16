//! The address type and mainnet of the chain being indexed. `bitcoin-dogecoin` keeps Bitcoin's
//! `Address` and `Network` (so its Bitcoin encodings would still compile) and adds Dogecoin's in
//! `dogecoin::`.

#[cfg(not(feature = "dogecoin"))]
pub use ::bitcoin::{Address as ChainAddress, Network as ChainNetwork};
#[cfg(not(feature = "dogecoin"))]
pub const MAINNET: ChainNetwork = ChainNetwork::Bitcoin;

#[cfg(feature = "dogecoin")]
pub use ::bitcoin::dogecoin::{Address as ChainAddress, Network as ChainNetwork};
#[cfg(feature = "dogecoin")]
pub const MAINNET: ChainNetwork = ChainNetwork::Dogecoin;

#[cfg(all(test, feature = "dogecoin"))]
mod tests {
    use ::bitcoin::{PubkeyHash, ScriptBuf, ScriptHash, hashes::Hash};

    use crate::{Address, AddressBytes};

    #[test]
    fn dogecoin_prefixes_round_trip() {
        let p2pkh = ScriptBuf::new_p2pkh(&PubkeyHash::from_byte_array([7; 20]));
        let p2sh = ScriptBuf::new_p2sh(&ScriptHash::from_byte_array([7; 20]));
        for (script, prefixes) in [(p2pkh, ['D', 'D']), (p2sh, ['9', 'A'])] {
            let address = Address::try_from(&script).unwrap();
            assert!(
                prefixes.iter().any(|&p| address.starts_with(p)),
                "{address:?}"
            );
            assert_eq!(AddressBytes::addr_to_script(&address).unwrap(), script);
        }
    }
}
