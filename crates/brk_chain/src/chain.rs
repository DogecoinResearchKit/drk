use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The blockchain this BRK instance is connected to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    #[default]
    Bitcoin,
    Litecoin,
    Dogecoin,
}

impl Chain {
    pub fn constants(self) -> ChainConstants {
        match self {
            Chain::Bitcoin => ChainConstants::BITCOIN,
            Chain::Litecoin => ChainConstants::LITECOIN,
            Chain::Dogecoin => ChainConstants::DOGECOIN,
        }
    }

    /// Returns false when the on-chain price oracle is not yet calibrated for
    /// this chain. Exchange-rate data from `brk_fetcher` is still available.
    pub fn supports_oracle(self) -> bool {
        matches!(self, Chain::Bitcoin)
    }

    /// Consensus block subsidy in the smallest unit (sats/koinu) expected at
    /// `height` under each chain's issuance schedule.
    ///
    /// Returns `None` when the expected subsidy is not deterministic — Dogecoin
    /// used randomized rewards before the simplified-reward fork at height
    /// 145,000, so there is no fixed "expected" value to compare actual issuance
    /// against. Callers computing an "unclaimed" delta should treat `None` as
    /// zero unclaimed.
    pub fn expected_block_subsidy(self, height: u32) -> Option<u64> {
        let c = self.constants();
        match self {
            // Geometric halving: initial_subsidy >> (height / blocks_per_halving).
            Chain::Bitcoin | Chain::Litecoin => {
                let halvings = height / c.blocks_per_halving;
                Some(if halvings >= 64 {
                    0
                } else {
                    c.initial_subsidy >> halvings
                })
            }
            // Dogecoin (matches Dogecoin Core `GetDogecoinBlockSubsidy`):
            //   height <   145,000: randomized reward -> None (no fixed expected)
            //   height <   600,000: (500,000 >> (height / 100,000)) DOGE
            //   height >=  600,000: fixed 10,000 DOGE
            Chain::Dogecoin => {
                const COIN: u64 = 100_000_000;
                if height < 145_000 {
                    None
                } else if height < 600_000 {
                    Some((500_000u64 >> (height / 100_000)) * COIN)
                } else {
                    Some(10_000 * COIN)
                }
            }
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Bitcoin => f.write_str("bitcoin"),
            Chain::Litecoin => f.write_str("litecoin"),
            Chain::Dogecoin => f.write_str("dogecoin"),
        }
    }
}

impl std::str::FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bitcoin" | "btc" => Ok(Chain::Bitcoin),
            "litecoin" | "ltc" => Ok(Chain::Litecoin),
            "dogecoin" | "doge" => Ok(Chain::Dogecoin),
            other => Err(format!(
                "unknown chain '{other}'; expected 'bitcoin', 'litecoin' or 'dogecoin'"
            )),
        }
    }
}

/// Protocol and deployment constants that differ between chains.
#[derive(Debug, Clone, Copy)]
pub struct ChainConstants {
    /// Number of blocks between reward halvings (BTC: 210,000 · LTC: 840,000).
    pub blocks_per_halving: u32,
    /// Number of blocks per difficulty-retarget epoch (both chains: 2,016).
    pub blocks_per_diff_epoch: u32,
    /// Target seconds per block (BTC: 600 · LTC: 150).
    pub seconds_per_block: u64,
    /// Initial block subsidy in smallest coin units (both chains: 50 × 10⁸).
    pub initial_subsidy: u64,
    /// Approximate blocks mined per year derived from `seconds_per_block`.
    pub blocks_per_year: u64,
    /// Unix timestamp of the genesis block.
    pub genesis_timestamp: u32,
    /// Unix timestamp of the index epoch start: January 1 of the genesis year
    /// (Bitcoin: 2009-01-01, Litecoin: 2011-01-01). Date-based indexes anchor
    /// here, so it must line up with `genesis_year`'s January — not the exact
    /// genesis block date — or the date↔index round-trip desyncs.
    pub index_epoch: u32,
    /// Calendar year of the genesis block (used for cohort start year).
    pub genesis_year: u16,
    /// Default JSON-RPC port for the node daemon.
    pub default_rpc_port: u16,
    /// Default data directory name segment on Linux (`~/<segment>`).
    pub default_datadir_linux: &'static str,
    /// Default data directory path segment on macOS
    /// (`~/Library/Application Support/<segment>`).
    pub default_datadir_mac: &'static str,
    /// Short ticker symbol.
    pub ticker: &'static str,
    /// Coin name for display.
    pub coin_name: &'static str,
    /// BIP30 duplicate coinbase `(duplicate_height, original_height)` pairs.
    /// The duplicate block contains a coinbase txid that collides with the one
    /// at `original_height`. Empty for chains that have no BIP30 duplicates.
    pub bip30_duplicate_heights: &'static [(u32, u32)],
    /// Binance spot symbol (e.g. `"BTCUSDT"`, `"LTCUSDT"`).
    pub binance_symbol: &'static str,
    /// Kraken OHLC endpoint pair name (e.g. `"XBTUSD"`, `"LTCUSD"`).
    pub kraken_pair: &'static str,
    /// Kraken result-object key in the OHLC response (e.g. `"XXBTZUSD"`, `"XLTCZUSD"`).
    pub kraken_result_key: &'static str,
    /// Coinbase Exchange product id for daily-history backfill (e.g. `"BTC-USD"`,
    /// `"LTC-USD"`). Unlike Binance/Kraken, Coinbase serves full paginated daily
    /// history, so it is the primary source for chains without an on-chain oracle.
    pub coinbase_product: &'static str,
    /// Bitfinex candle symbol (e.g. `"tBTCUSD"`, `"tLTCUSD"`). Bitfinex serves
    /// daily history back to 2013 (earlier than Coinbase's 2016 listing) in a
    /// single request, so it fills the early-history gap for exchange pricing.
    pub bitfinex_symbol: &'static str,
    /// Calendar month (1 = Jan) of the `index_epoch` date. Used by client generators.
    pub index_epoch_month: u8,
    /// Calendar day of the `index_epoch` date. Used by client generators.
    pub index_epoch_day: u8,
}

impl ChainConstants {
    pub const BITCOIN: Self = Self {
        blocks_per_halving: 210_000,
        blocks_per_diff_epoch: 2_016,
        seconds_per_block: 600,
        initial_subsidy: 50 * 100_000_000,
        blocks_per_year: 52_560,
        genesis_timestamp: 1_231_006_505,
        index_epoch: 1_230_768_000, // 2009-01-01 00:00:00 UTC
        genesis_year: 2009,
        default_rpc_port: 8332,
        default_datadir_linux: ".bitcoin",
        default_datadir_mac: "Bitcoin",
        ticker: "BTC",
        coin_name: "Bitcoin",
        bip30_duplicate_heights: &[
            (91_842, 91_812), // block 91_842 coinbase duplicates 91_812
            (91_880, 91_722), // block 91_880 coinbase duplicates 91_722
        ],
        binance_symbol: "BTCUSDT",
        kraken_pair: "XBTUSD",
        kraken_result_key: "XXBTZUSD",
        coinbase_product: "BTC-USD",
        bitfinex_symbol: "tBTCUSD",
        index_epoch_month: 1, // 2009-01-01
        index_epoch_day: 1,
    };

    pub const LITECOIN: Self = Self {
        blocks_per_halving: 840_000,
        blocks_per_diff_epoch: 2_016,
        seconds_per_block: 150,
        initial_subsidy: 50 * 100_000_000,
        blocks_per_year: 210_240,
        genesis_timestamp: 1_317_972_665,
        index_epoch: 1_293_840_000, // 2011-01-01 00:00:00 UTC (Jan 1 of genesis year)
        genesis_year: 2011,
        default_rpc_port: 9332,
        default_datadir_linux: ".litecoin",
        default_datadir_mac: "Litecoin",
        ticker: "LTC",
        coin_name: "Litecoin",
        bip30_duplicate_heights: &[],
        binance_symbol: "LTCUSDT",
        kraken_pair: "LTCUSD",
        kraken_result_key: "XLTCZUSD",
        coinbase_product: "LTC-USD",
        bitfinex_symbol: "tLTCUSD",
        index_epoch_month: 1, // 2011-01-01
        index_epoch_day: 1,
    };

    pub const DOGECOIN: Self = Self {
        // TODO(dogecoin): Dogecoin has no perpetual halving. Its subsidy follows
        // a step-down schedule that settles at a fixed 10,000 DOGE from block
        // 600,000 onward. `blocks_per_halving` is a non-zero placeholder (used as
        // a divisor by the halving index) until the step-down subsidy model and
        // halving-cohort handling are ported in `brk_computer`.
        blocks_per_halving: 600_000,
        // TODO(dogecoin): pre-Digishield retargeted every 240 blocks; from block
        // 145,000 (Digishield) difficulty retargets every block. `1` reflects the
        // current per-block retarget; revisit alongside difficulty metrics.
        blocks_per_diff_epoch: 1,
        seconds_per_block: 60,
        // TODO(dogecoin): early rewards were randomized (0–1,000,000 DOGE) then
        // stepped down. This placeholder is the perpetual reward in koinu; the
        // real per-height schedule is handled in the deferred subsidy port.
        initial_subsidy: 10_000 * 100_000_000,
        blocks_per_year: 525_600, // 1440 blocks/day * 365
        genesis_timestamp: 1_386_325_540, // 2013-12-06 Dogecoin mainnet genesis
        index_epoch: 1_356_998_400, // 2013-01-01 00:00:00 UTC (Jan 1 of genesis year)
        genesis_year: 2013,
        default_rpc_port: 22555,
        default_datadir_linux: ".dogecoin",
        default_datadir_mac: "Dogecoin",
        ticker: "DOGE",
        coin_name: "Dogecoin",
        bip30_duplicate_heights: &[],
        // TODO(dogecoin): exchange symbols below are best-effort and unused while
        // the price fetcher/oracle stays Bitcoin-only; verify before enabling
        // DOGE/USD pricing (Kraken uses the `XDG` asset code for Dogecoin).
        binance_symbol: "DOGEUSDT",
        kraken_pair: "XDGUSD",
        kraken_result_key: "XXDGZUSD",
        coinbase_product: "DOGE-USD",
        bitfinex_symbol: "tDOGE:USD",
        index_epoch_month: 1, // 2013-01-01
        index_epoch_day: 1,
    };
}
