use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::Sats;
use vecdb::{Exit, VecIndex};

use super::Vecs;
use crate::{mining, outputs, price};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        outputs: &outputs::Vecs,
        mining: &mining::Vecs,
        prices: &price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        // The genesis coinbase is unspendable on every chain, so its subsidy is
        // permanently burned. Use the chain's expected genesis subsidy (50 BTC
        // for Bitcoin/Litecoin; Dogecoin's genesis predates its deterministic
        // schedule, so this contributes zero — negligible against total supply).
        let genesis_burned = Sats::new(indexer.chain.expected_block_subsidy(0).unwrap_or(0));
        self.total
            .compute_with(starting_height, prices, exit, |sats| {
                Ok(sats.compute_transform2(
                    starting_height,
                    &outputs.value.op_return.block.sats,
                    &mining.rewards.unclaimed.block.sats,
                    |(h, op_return, unclaimed, ..)| {
                        let genesis = if h.to_usize() == 0 {
                            genesis_burned
                        } else {
                            Sats::ZERO
                        };
                        (h, genesis + op_return + unclaimed)
                    },
                    exit,
                )?)
            })?;

        Ok(())
    }
}
