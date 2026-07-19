use std::{
    collections::BTreeMap,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use brk_error::{Error, Result};
use brk_types::{Cents, Close, Date, Dollars, High, Low, OHLCCents, Open, Timestamp};
use serde_json::Value;
use tracing::info;
use ureq::Agent;

use crate::{checked_get, ohlc::date_from_timestamp};

/// CryptoCompare's `histoday` endpoint returns at most 2000 candles per
/// request, so history is backfilled one 2000-day window at a time, walking
/// backward via `toTs`.
const LIMIT: i64 = 2000;
const DAY_SECS: i64 = 86_400;

/// Environment variable holding the CryptoCompare (CoinDesk) API key. The
/// free/no-key `histoday` endpoint was retired, so a key is required for the
/// deep daily history that fills the pre-2021 gap.
pub const API_KEY_ENV: &str = "CRYPTOCOMPARE_API_KEY";

/// CryptoCompare daily-OHLC source. Its aggregated index reaches back to a
/// coin's earliest trading (DOGE/USD to ~2014), filling the pre-2021 gap left
/// by Coinbase and Bitfinex, whose DOGE markets only list from 2021. Used as
/// the base layer for chains without BRK's on-chain oracle; more liquid
/// exchange sources overlay it on recent dates.
#[derive(Clone)]
pub struct CryptoCompare {
    agent: Agent,
    /// From-symbol, e.g. `"DOGE"`.
    fsym: &'static str,
    /// To-symbol, e.g. `"USD"`.
    tsym: &'static str,
    /// Unix seconds to stop backfilling at (typically the chain's genesis).
    start_unix: i64,
    /// API key; `None` disables the source (returns an error on fetch).
    api_key: Option<String>,
    _1d: Option<BTreeMap<Date, OHLCCents>>,
}

impl CryptoCompare {
    pub fn new_with_agent(
        agent: Agent,
        fsym: &'static str,
        tsym: &'static str,
        start_unix: i64,
        api_key: Option<String>,
    ) -> Self {
        Self {
            agent,
            fsym,
            tsym,
            start_unix,
            api_key,
            _1d: None,
        }
    }

    /// Read the API key from the environment (`CRYPTOCOMPARE_API_KEY`).
    pub fn api_key_from_env() -> Option<String> {
        std::env::var(API_KEY_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Full daily OHLC history, paged backward from now in 2000-candle windows
    /// until we reach `start_unix`, run out of history, or hit an all-zero
    /// (pre-listing) window.
    pub fn fetch_all_daily(&self) -> Result<BTreeMap<Date, OHLCCents>> {
        let api_key = self.api_key.as_deref().ok_or_else(|| {
            Error::NotFound(format!(
                "CryptoCompare API key not set (env {API_KEY_ENV}); skipping deep history"
            ))
        })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut out: BTreeMap<Date, OHLCCents> = BTreeMap::new();
        let mut to_ts = now;

        loop {
            let url = format!(
                "https://min-api.cryptocompare.com/data/v2/histoday?fsym={}&tsym={}&limit={LIMIT}&toTs={to_ts}&api_key={api_key}",
                self.fsym, self.tsym
            );
            info!(
                "Fetching https://min-api.cryptocompare.com/data/v2/histoday?fsym={}&tsym={}&limit={LIMIT}&toTs={to_ts} ...",
                self.fsym, self.tsym
            );

            let bytes = checked_get(&self.agent, &url)?;
            let json: Value = serde_json::from_slice(&bytes)?;

            if json.get("Response").and_then(|v| v.as_str()) != Some("Success") {
                let msg = json
                    .get("Message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(Error::Parse(format!("CryptoCompare: {msg}")));
            }

            // `Data.Data` is an array of daily bars ascending by time; pre-listing
            // days are padded with all-zero OHLC.
            let arr = json
                .get("Data")
                .and_then(|d| d.get("Data"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| Error::Parse(format!("CryptoCompare: unexpected response {json}")))?;
            if arr.is_empty() {
                break;
            }

            let mut earliest = i64::MAX;
            let mut nonzero_in_window = false;
            for c in arr {
                let ts = c.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
                earliest = earliest.min(ts);
                let f = |k: &str| c.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let close = f("close");
                if close == 0.0 {
                    continue; // pre-listing padding
                }
                nonzero_in_window = true;
                let cents = |v: f64| Cents::from(Dollars::from(v));
                let date = date_from_timestamp(Timestamp::from(ts as u32));
                out.insert(
                    date,
                    OHLCCents::from((
                        Open::new(cents(f("open"))),
                        High::new(cents(f("high"))),
                        Low::new(cents(f("low"))),
                        Close::new(cents(close)),
                    )),
                );
            }

            if earliest == i64::MAX || earliest <= self.start_unix || !nonzero_in_window {
                break;
            }
            let next_to = earliest - DAY_SECS;
            if next_to >= to_ts {
                break; // no forward progress; avoid looping forever
            }
            to_ts = next_to;
            // Stay under CryptoCompare's public rate limit.
            sleep(Duration::from_millis(250));
        }

        if out.is_empty() {
            return Err(Error::NotFound(format!(
                "CryptoCompare returned no candles for {}/{}",
                self.fsym, self.tsym
            )));
        }

        Ok(out)
    }

    /// Full daily close-price history as `Date -> Cents`, loaded (and cached) on
    /// first call.
    pub fn daily_closes(&mut self) -> Result<BTreeMap<Date, Cents>> {
        if self._1d.is_none() {
            self._1d.replace(self.fetch_all_daily()?);
        }
        Ok(self
            ._1d
            .as_ref()
            .unwrap()
            .iter()
            .map(|(date, ohlc)| (*date, *ohlc.close))
            .collect())
    }
}
