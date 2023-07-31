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
    public_call::call_public_function_internal,
    arguments,
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

// An open function doesn't have a context, so we call this pure function instead of going via Context.
fn call_public_function<N>(
    contract_address: Field, 
    function_selector: Field,
    args: [Field; N],
) -> [Field; RETURN_VALUES_LENGTH] {
    let args_hash = abi::hash_args(args);
    assert(args_hash == arguments::pack_arguments(args));
    call_public_function_internal(
        contract_address, 
        function_selector, 
        args_hash,
    )
}

fn call_public_function_no_args(
    contract_address: Field, 
    function_selector: Field,
) -> [Field; RETURN_VALUES_LENGTH] {
    call_public_function_internal(
        contract_address, 
        function_selector, 
        0,
    )
}
