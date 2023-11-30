use crate::transaction::context::TxContext;
use crate::abis::block_header::BlockHeader;

struct CombinedConstantData {
    block_header: BlockHeader,
    tx_context: TxContext,
}
