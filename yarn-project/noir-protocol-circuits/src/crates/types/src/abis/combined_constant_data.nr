use crate::transaction::context::TxContext;
use crate::abis::historical_block_data::HistoricalBlockData;

struct CombinedConstantData {
    block_data: HistoricalBlockData,
    tx_context: TxContext,
}
