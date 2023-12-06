use crate::{
    abi,
    hash::pedersen_hash,
    abi::{
        PublicCircuitPublicInputs,
        FunctionData,
    },
};
use dep::protocol_types::constants::{
    RETURN_VALUES_LENGTH,
    GENERATOR_INDEX__CALL_STACK_ITEM,
};

// oracles
use crate::oracle::{
    enqueue_public_function_call::enqueue_public_function_call_internal,
};

struct PublicCallStackItem {
    contract_address: Field,
    function_data: FunctionData,
    public_inputs: PublicCircuitPublicInputs,
    is_execution_request: bool,
}

impl PublicCallStackItem {
    pub fn hash(self) -> Field {
        pedersen_hash([
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)
    }
}

