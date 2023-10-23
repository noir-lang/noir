use crate::abi::FunctionData;
use crate::abi::PrivateCircuitPublicInputs;
use crate::constants_gen::GENERATOR_INDEX__CALL_STACK_ITEM;
use crate::hash::pedersen_hash;

struct PrivateCallStackItem {
    contract_address: Field,
    function_data: FunctionData,
    public_inputs: PrivateCircuitPublicInputs,
    is_execution_request: bool,
}

impl PrivateCallStackItem {
    pub fn hash(self) -> Field {
        pedersen_hash([
            self.contract_address,
            self.function_data.hash(),
            self.public_inputs.hash(),
        ], GENERATOR_INDEX__CALL_STACK_ITEM)
    }
}