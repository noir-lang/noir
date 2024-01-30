use crate::transaction::context::TxContext;
use crate::header::Header;

struct CombinedConstantData {
    header: Header,
    tx_context: TxContext,
}
