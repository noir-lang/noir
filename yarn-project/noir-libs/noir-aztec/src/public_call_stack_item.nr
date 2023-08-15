use crate::{
    abi,
    abi::{
        PublicCircuitPublicInputs,
        FunctionData,
    },
};
use crate::constants_gen::{
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
    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator([
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)[0]
    }
}

