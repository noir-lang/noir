use crate::abis::{
    function_data::FunctionData,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_circuit_public_inputs::PublicCircuitPublicInputs,
};
use crate::address::AztecAddress;
use crate::constants::{
    GENERATOR_INDEX__CALL_STACK_ITEM,
};

struct PrivateCallStackItem {
    // This is the _actual_ contract address relating to where this function's code resides in the
    // contract tree. Regardless of whether this is a call or delegatecall, this
    // `contract_address` _does not change_. Amongst other things, it's used as a lookup for
    // getting the correct code from the tree. There is a separate `storage_contract_address`
    // within a CallStackItem which varies depending on whether this is a call or delegatecall.
    contract_address: AztecAddress,
    public_inputs: PrivateCircuitPublicInputs,
    function_data: FunctionData,
    // Not really needed for PrivateCallStackItem.
    is_execution_request: bool,
}

impl PrivateCallStackItem {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.contract_address.to_field(),
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)
    }
}

struct PublicCallStackItem {
    contract_address: AztecAddress,
    public_inputs: PublicCircuitPublicInputs,
    function_data: FunctionData,
    // True if this call stack item represents a request to execute a function rather than a
    // fulfilled execution. Used when enqueuing calls from private to public functions.
    is_execution_request: bool,
}

impl PublicCallStackItem {
    fn hash(self) -> Field {
        let item = if self.is_execution_request {
            self.as_execution_request()
        } else {
            self
        };

        dep::std::hash::pedersen_hash_with_separator([
            item.contract_address.to_field(),
            item.function_data.hash(),
            item.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)
    }

    fn as_execution_request(self) -> Self {
        let public_inputs = self.public_inputs;
        let mut request_public_inputs: PublicCircuitPublicInputs = dep::std::unsafe::zeroed();
        request_public_inputs.call_context = public_inputs.call_context;
        request_public_inputs.args_hash = public_inputs.args_hash;

        PublicCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            is_execution_request: true,
            public_inputs: request_public_inputs,
        }
    }
}
