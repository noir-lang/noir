use dep::aztec::constants_gen;
use crate::address::Address;
use crate::transaction::context::TxContext;
use crate::abis::function_data::FunctionData;

struct TxRequest {
    origin : Address,
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
        ], constants_gen::GENERATOR_INDEX__TX_REQUEST)
    }
}
