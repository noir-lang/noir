use crate::abi::FunctionData;
use crate::abi::PrivateCircuitPublicInputs;
use crate::constants_gen::GENERATOR_INDEX__CALL_STACK_ITEM;

struct PrivateCallStackItem {
    contract_address: Field,
    function_data: FunctionData,
    public_inputs: PrivateCircuitPublicInputs,
    is_execution_request: bool,
}

impl PrivateCallStackItem {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator([
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)[0]
    }
}