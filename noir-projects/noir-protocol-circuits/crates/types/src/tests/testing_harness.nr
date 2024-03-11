use crate::{tests::fixtures, transaction::tx_context::TxContext, address::compute_initialization_hash};

pub fn build_tx_context() -> TxContext {
    TxContext { is_fee_payment_tx: false, is_rebate_payment_tx: false, chain_id: 1, version: 0 }
}
