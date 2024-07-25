use crate::abis::{
    function_data::FunctionData, public_circuit_public_inputs::PublicCircuitPublicInputs,
    public_call_stack_item_compressed::PublicCallStackItemCompressed
};
use crate::address::AztecAddress;

struct PublicCallStackItem {
    contract_address: AztecAddress,
    public_inputs: PublicCircuitPublicInputs,
    function_data: FunctionData,
    // True if this call stack item represents a request to execute a function rather than a
    // fulfilled execution. Used when enqueuing calls from private to public functions.
    is_execution_request: bool,
}

impl PublicCallStackItem {
    fn as_execution_request(self) -> Self {
        let public_inputs = self.public_inputs;
        let mut request_public_inputs = PublicCircuitPublicInputs::empty();
        request_public_inputs.call_context = public_inputs.call_context;
        request_public_inputs.args_hash = public_inputs.args_hash;

        let call_stack_item = PublicCallStackItem {
            contract_address: self.contract_address,
            function_data: self.function_data,
            is_execution_request: true,
            public_inputs: request_public_inputs
        };
        call_stack_item
    }

    fn get_compressed(self) -> PublicCallStackItemCompressed {
        let item = if self.is_execution_request {
            self.as_execution_request()
        } else {
            self
        };

        PublicCallStackItemCompressed {
            contract_address: item.contract_address,
            call_context: item.public_inputs.call_context,
            args_hash: item.public_inputs.args_hash,
            returns_hash: item.public_inputs.returns_hash,
            revert_code: item.public_inputs.revert_code,
            start_gas_left: item.public_inputs.start_gas_left,
            end_gas_left: item.public_inputs.end_gas_left
        }
    }
}
