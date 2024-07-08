use crate::abis::{
    function_data::FunctionData, public_circuit_public_inputs::PublicCircuitPublicInputs,
    public_call_stack_item_compressed::PublicCallStackItemCompressed
};
use crate::address::AztecAddress;
use crate::constants::GENERATOR_INDEX__CALL_STACK_ITEM;
use crate::traits::Hash;

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
        // WARNING: if updating, see comment in public_call_stack_item.ts's `PublicCallStackItem.getCompressed()`
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
            function_data: item.function_data,
            args_hash: item.public_inputs.args_hash,
            returns_hash: item.public_inputs.returns_hash,
            revert_code: item.public_inputs.revert_code,
            start_gas_left: item.public_inputs.start_gas_left,
            end_gas_left: item.public_inputs.end_gas_left
        }
    }
}

mod tests {
    use crate::{
        abis::{
        function_data::FunctionData, function_selector::FunctionSelector, note_hash::NoteHash,
        public_circuit_public_inputs::PublicCircuitPublicInputs,
        public_call_stack_item::PublicCallStackItem
    },
        address::AztecAddress, constants::GENERATOR_INDEX__CALL_STACK_ITEM, traits::Hash
    };

    #[test]
    fn compute_call_stack_item_request_hash() {
        let contract_address = AztecAddress::from_field(1);
        let function_data = FunctionData { selector: FunctionSelector::from_u32(2), is_private: false };

        let mut public_inputs = PublicCircuitPublicInputs::empty();
        public_inputs.note_hashes[0] = NoteHash {
            value: 1,
            counter: 0,
        };

        let call_stack_item = PublicCallStackItem { contract_address, public_inputs, is_execution_request: true, function_data };

        // Value from public_call_stack_item.test.ts "Computes a callstack item request hash" test
        let test_data_call_stack_item_request_hash = 0x2268218e4dd9e61345f407816d63de9c7a83b031b7820a17470748bae488fb50;
        assert_eq(call_stack_item.get_compressed().hash(), test_data_call_stack_item_request_hash);
    }

    #[test]
    fn compute_call_stack_item_hash() {
        let contract_address = AztecAddress::from_field(1);
        let function_data = FunctionData { selector: FunctionSelector::from_u32(2), is_private: false };

        let mut public_inputs = PublicCircuitPublicInputs::empty();
        public_inputs.note_hashes[0] = NoteHash {
            value: 1,
            counter: 0,
        };

        let call_stack_item = PublicCallStackItem { contract_address, public_inputs, is_execution_request: false, function_data };

        // Value from public_call_stack_item.test.ts "Computes a callstack item hash" test
        let test_data_call_stack_item_hash = 0x2268218e4dd9e61345f407816d63de9c7a83b031b7820a17470748bae488fb50;
        assert_eq(call_stack_item.get_compressed().hash(), test_data_call_stack_item_hash);
    }
}
