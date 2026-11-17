use bitview_primitives::{CentsSquaredSats, CoinDays, Count};
use brk_types::{Cents, CentsSats, CentsSigned, CentsTotal, Sats, SatsTotal};
use derive_more::AddAssign;

/// Additive accounting inputs from disjoint age bands at one height.
#[derive(Clone, Copy, Default, AddAssign)]
pub(crate) struct AccountingTotals {
    pub supply: Sats,
    pub count: Count,
    pub spent_count: Count,
    pub volume_sats: SatsTotal,
    pub volume_cents: CentsTotal,
    pub volume_profit_sats: SatsTotal,
    pub volume_profit_cents: CentsTotal,
    pub volume_loss_sats: SatsTotal,
    pub volume_loss_cents: CentsTotal,
    pub cdd: CoinDays,
    pub cap_raw: CentsSats,
    pub capitalized_cap_raw: CentsSquaredSats,
    pub profit: Cents,
    pub loss: Cents,
    pub net_pnl: CentsSigned,
    pub value_destroyed: Cents,
    pub unrealized_profit: Cents,
    pub unrealized_loss: Cents,
    pub supply_profit: Sats,
    pub supply_loss: Sats,
    pub capital_profit: Cents,
    pub capital_loss: Cents,
    pub peak_regret_raw: CentsSats,
}
