use brk_types::{Bitcoin, MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Weight};
use serde::Deserialize;

/// The subset of a `getmempoolentry` response that BRK consumes.
#[derive(Deserialize)]
#[cfg_attr(feature = "dogecoin", serde(try_from = "dogecoin::Entry"))]
pub struct MempoolEntry {
    vsize: VSize,
    weight: Weight,
    time: Timestamp,
    fees: MempoolFees,
}

impl MempoolEntry {
    pub fn into_info(self, txid: Txid) -> MempoolEntryInfo {
        MempoolEntryInfo {
            txid,
            vsize: self.vsize,
            weight: self.weight,
            fee: Sats::from(self.fees.base),
            first_seen: self.time,
        }
    }
}

#[derive(Deserialize)]
struct MempoolFees {
    base: Bitcoin,
}

/// Dogecoin Core 1.14 answers as Bitcoin Core 0.14 did: `size` (no witness, so the vsize) and
/// `fee`, without `vsize`, `weight` or `fees`. Later answers keep their own fields.
#[cfg(feature = "dogecoin")]
mod dogecoin {
    use super::*;

    #[derive(Deserialize)]
    pub struct Entry {
        vsize: Option<VSize>,
        size: Option<VSize>,
        weight: Option<Weight>,
        time: Timestamp,
        fees: Option<MempoolFees>,
        fee: Option<Bitcoin>,
    }

    impl TryFrom<Entry> for MempoolEntry {
        type Error = &'static str;

        fn try_from(entry: Entry) -> Result<Self, Self::Error> {
            let vsize = entry.vsize.or(entry.size).ok_or("getmempoolentry: no size")?;
            let base = entry
                .fees
                .map(|fees| fees.base)
                .or(entry.fee)
                .ok_or("getmempoolentry: no fee")?;
            Ok(Self {
                vsize,
                weight: entry.weight.unwrap_or_else(|| Weight::from(vsize)),
                time: entry.time,
                fees: MempoolFees { base },
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use serde_json::from_str;

        use super::*;

        /// `getmempoolentry` as Dogecoin Core 1.14.9 answers it.
        const DOGECOIN_1_14: &str = r#"{"size":226,"fee":0.01,"modifiedfee":0.01,"time":1700000000,
            "height":5000000,"startingpriority":0,"currentpriority":0,"descendantcount":1,
            "descendantsize":226,"descendantfees":1000000,"ancestorcount":1,"ancestorsize":226,
            "ancestorfees":1000000,"depends":[]}"#;

        #[test]
        fn reads_dogecoin_core_1_14() {
            let entry: MempoolEntry = from_str(DOGECOIN_1_14).unwrap();
            assert_eq!(entry.vsize, VSize::from(226_usize));
            assert_eq!(entry.weight, Weight::from(904_usize));
            assert_eq!(Sats::from(entry.fees.base), Sats::new(1_000_000));
        }

        #[test]
        fn reads_later_cores() {
            let json = r#"{"vsize":141,"weight":561,"time":1700000000,"fees":{"base":0.02}}"#;
            let entry: MempoolEntry = from_str(json).unwrap();
            assert_eq!(entry.vsize, VSize::from(141_usize));
            assert_eq!(entry.weight, Weight::from(561_usize));
            assert_eq!(Sats::from(entry.fees.base), Sats::new(2_000_000));
        }
    }
}
