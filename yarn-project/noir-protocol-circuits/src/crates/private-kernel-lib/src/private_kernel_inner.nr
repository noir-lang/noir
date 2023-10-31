use dep::aztec::constants_gen::{MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX};
use crate::common;
use crate::mocked::{Proof, AggregationObject, verify_previous_kernel_state};
use crate::transaction::request::TxRequest;
use crate::abis::{
    combined_constant_data::CombinedConstantData,
    previous_kernel_data::PreviousKernelData,
    private_kernel::private_call_data::PrivateCallData,
    new_contract_data::NewContractData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder}
};
use crate::utils::{
    arrays,
    bounded_vec::BoundedVec,
};
use dep::std::unsafe;

struct PrivateKernelInputsInner {
    previous_kernel : PreviousKernelData,
    private_call : PrivateCallData,
}

impl PrivateKernelInputsInner {
    // TODO: We can create a type alias for private call stack item
    fn pop_and_validate_this_private_call_hash(self, public_inputs : &mut KernelCircuitPublicInputsBuilder){
        let private_call = self.private_call;
        let popped_private_call_hash = public_inputs.end.private_call_stack.pop();

        let calculated_this_private_call_hash = private_call.call_stack_item.hash();

        assert(popped_private_call_hash == calculated_this_private_call_hash, "calculated private_call_hash does not match provided private_call_hash at the top of the callstack");
    }

    fn validate_contract_tree_root(self) {
        let purported_contract_tree_root = self.private_call.call_stack_item.public_inputs().historical_block_data.contract_tree_root();
        let previous_kernel_contract_tree_root = self.previous_kernel.public_inputs.constants.block_data.contract_tree_root();

        assert(purported_contract_tree_root == previous_kernel_contract_tree_root, "purported_contract_tree_root doesn't match previous_kernel_contract_tree_root");
    }

    fn validate_inputs(self) {
        let this_call_stack_item = self.private_call.call_stack_item;
        assert(this_call_stack_item.function_data().is_private, "Cannot execute a non-private function with the private kernel circuit");

        let start = self.previous_kernel.public_inputs.end;
        let start_private_call_stack_length = arrays::array_length(start.private_call_stack);

        assert(self.previous_kernel.public_inputs.is_private, "Cannot verify a non-private kernel snark in the private kernel circuit");
        assert(this_call_stack_item.function_data().is_constructor == false, "A constructor must be executed as the first tx in the recursion");
        assert(start_private_call_stack_length != 0, "Cannot execute private kernel circuit with an empty private call stack");
    }

    fn native_private_kernel_circuit_inner(self) -> KernelCircuitPublicInputs {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;

        common::validate_previous_kernel_values(self.previous_kernel.public_inputs.end);

        // Do this before any functions can modify the inputs.
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        self.validate_inputs();

        common::validate_arrays(self.private_call.call_stack_item.public_inputs());

        self.pop_and_validate_this_private_call_hash(&mut public_inputs);

        common::validate_call_stack(self.private_call);

        common::validate_read_requests(
            public_inputs.constants.block_data.note_hash_tree_root(),
            self.private_call.call_stack_item.public_inputs().read_requests,  // read requests from private call
            self.private_call.read_request_membership_witnesses);

        //TODO(David): feels like update_end_values should happen later
        common::update_end_values(self.private_call, &mut public_inputs);

        // ensure that historic/purported contract tree root matches the one in previous kernel
        self.validate_contract_tree_root();

        let private_call_stack_item = self.private_call.call_stack_item;

        common::contract_logic(self.private_call,
                          &mut public_inputs,
                          private_call_stack_item.public_inputs().contract_deployment_data,
                          private_call_stack_item.function_data());

        let (is_previous_state_valid, updated_aggregation_object) = verify_previous_kernel_state(public_inputs.end.aggregation_object,self.private_call.proof);
        assert(is_previous_state_valid);

        public_inputs.end.aggregation_object = updated_aggregation_object;

        public_inputs.finish()
    }
}
