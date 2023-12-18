use crate::constants::GENERATOR_INDEX__TX_REQUEST;
use crate::address::AztecAddress;
use crate::transaction::context::TxContext;
use crate::abis::function_data::FunctionData;

struct TxRequest {
    origin : AztecAddress,
    args_hash : Field,
    tx_context : TxContext,
    function_data : FunctionData
}

impl TxRequest {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.origin.to_field(),
            self.function_data.hash(),
            self.args_hash,
            self.tx_context.hash(),
        ], GENERATOR_INDEX__TX_REQUEST)
    }
}
