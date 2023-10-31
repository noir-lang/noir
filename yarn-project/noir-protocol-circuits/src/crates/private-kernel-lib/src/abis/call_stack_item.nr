use crate::address::Address;
use crate::abis::function_data::FunctionData;
use dep::aztec::constants_gen::{MAX_READ_REQUESTS_PER_CALL};
use crate::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs;
use crate::abis::public_circuit_public_inputs::PublicCircuitPublicInputs;
use dep::aztec::constants_gen;

// TODO(Noir-bug): These should be type aliases

struct PublicCallStackItem {
    inner : CallStackItem<PublicCircuitPublicInputs>
}

struct PrivateCallStackItem {
    inner : CallStackItem<PrivateCircuitPublicInputs>
}

// Parametrize function over the public inputs.
// The public inputs can be from either a public or a private circuit
// TODO: We could monomorphize this struct and create a PrivateCallStackItem and 
// TODO: a PublicCallStackItem
struct CallStackItem<AppCircuitPublicInputs> {
    // Comment copied from the cpp codebase
    //
    // This is the _actual_ contract address relating to where this function's code resides in the
    // contract tree. Regardless of whether this is a call or delegatecall, this
    // `contract_address` _does not change_. Amongst other things, it's used as a lookup for
    // getting the correct code from the tree. There is a separate `storage_contract_address`
    // within a CallStackItem which varies depending on whether this is a call or delegatecall.
    contract_address : Address,

    public_inputs : AppCircuitPublicInputs,

    // True if this call stack item represents a request to execute a function rather than a
    // fulfilled execution. Used when enqueuing calls from private to public functions.
    is_execution_request : bool,

    function_data : FunctionData,
}

// TODO(Traits): Defining a hash trait bound
// Would allow the impl to be over `T : Hash`
// Where T is public inputs

// Noir(bug): If we can use a type alias instead of a new struct
// then we can remove the getter methods below for things like
// public_inputs and contract_address
impl PublicCallStackItem {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.inner.contract_address.to_field(),
            self.inner.function_data.hash(),
            self.inner.public_inputs.hash(),
        ], constants_gen::GENERATOR_INDEX__CALL_STACK_ITEM)
    }

    fn public_inputs(self) -> PublicCircuitPublicInputs {
        self.inner.public_inputs
    }

    fn contract_address(self) -> Address{
        self.inner.contract_address
    }

    fn function_data(self) -> FunctionData{
        self.inner.function_data
    }
}
impl PrivateCallStackItem {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.inner.contract_address.to_field(),
            self.inner.function_data.hash(),
            self.inner.public_inputs.hash(),
        ], constants_gen::GENERATOR_INDEX__CALL_STACK_ITEM)
    }

    fn public_inputs(self) -> PrivateCircuitPublicInputs {
        self.inner.public_inputs
    }

    fn contract_address(self) -> Address{
        self.inner.contract_address
    }

    fn function_data(self) -> FunctionData{
        self.inner.function_data
    }
}
