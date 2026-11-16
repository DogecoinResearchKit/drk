//! The fields Bitview reads of Bitcoin Core's RPC answers, under Core's names. serde skips
//! every other field, so Core adding or reshaping those changes nothing here.

use serde::Deserialize;

/// `getblockcount`: the best chain's height.
#[derive(Deserialize)]
pub struct GetBlockCount(pub u64);

/// `getblockheader`, verbose.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetBlockHeaderVerbose {
    /// Zero or less when the block is off the best chain.
    pub confirmations: i64,
    pub height: i64,
}

/// `getblocktemplate`.
#[derive(Deserialize)]
pub struct GetBlockTemplate {
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: String,
    pub transactions: Vec<BlockTemplateTransaction>,
}

/// One of `getblocktemplate`'s transactions. Dogecoin Core 1.14 can leave out the SegWit-era
/// `hash`, `sigops` and `weight`; [`BlockTemplateTransaction::fill_pre_segwit`] derives them.
#[derive(Deserialize)]
pub struct BlockTemplateTransaction {
    /// The serialized transaction, in hex.
    pub data: String,
    pub txid: String,
    /// The wtxid.
    #[cfg_attr(feature = "dogecoin", serde(default))]
    pub hash: String,
    /// The 1-based positions in the template of the transactions it spends.
    pub depends: Vec<i64>,
    pub fee: i64,
    #[cfg_attr(feature = "dogecoin", serde(default))]
    pub sigops: i64,
    #[cfg_attr(feature = "dogecoin", serde(default))]
    pub weight: u64,
}

#[cfg(feature = "dogecoin")]
impl BlockTemplateTransaction {
    /// Without witnesses the wtxid is the txid and the weight four times the size.
    pub fn fill_pre_segwit(&mut self) {
        if self.hash.is_empty() {
            self.hash.clone_from(&self.txid);
        }
        if self.weight == 0 {
            self.weight = 4 * (self.data.len() as u64 / 2);
        }
    }
}

/// `getmempoolinfo`.
#[derive(Deserialize)]
pub struct GetMempoolInfo {
    /// In BTC per kvB.
    #[serde(rename = "mempoolminfee")]
    pub mempool_min_fee: f64,
}

/// `getblockchaininfo`.
#[derive(Deserialize)]
pub struct GetBlockchainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
}
