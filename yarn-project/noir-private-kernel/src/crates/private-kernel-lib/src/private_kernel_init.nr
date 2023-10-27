use dep::aztec::constants_gen;
use crate::common;
use crate::mocked::{Proof, AggregationObject,verify_previous_kernel_state};
use crate::transaction::request::TxRequest;
use crate::abis::{
    combined_constant_data::CombinedConstantData,
    private_kernel::private_call_data::PrivateCallData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder}
};
use crate::utils::arrays;
use dep::std::unsafe;

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInputsInit {
    tx_request : TxRequest,
    private_call : PrivateCallData,
}

impl PrivateKernelInputsInit {
    fn initialize_end_values(self, public_inputs : &mut KernelCircuitPublicInputsBuilder) {
        let private_call_public_inputs = self.private_call.call_stack_item.public_inputs();
        let constants = CombinedConstantData{
            block_data : private_call_public_inputs.historical_block_data,
            tx_context : self.tx_request.tx_context,
        };

        public_inputs.constants = constants;
    }

    // Confirm that the TxRequest (user's intent) 
    // matches the private call being executed
    fn validate_this_private_call_against_tx_request(self) {
        let tx_request = self.tx_request;
        // Call stack item for the initial call
        let call_stack_item = self.private_call.call_stack_item;
        
        // Checks to ensure that the user's intent matches the initial private call
        //
        // We use the word correct to denote whether it matches the user intent.
        //
        // Ensure we are calling the correct initial contract
        let origin_address_matches = tx_request.origin.eq(call_stack_item.contract_address());
        assert(origin_address_matches, "origin address does not match call stack items contract address");
        //
        // Ensure we are calling the correct initial function in the contract
        let entry_point_function_matches = tx_request.function_data.hash() == call_stack_item.function_data().hash();
        assert(entry_point_function_matches, "tx_request.function_data must match call_stack_item.function_data");
        //
        // Ensure we are passing the correct arguments to the function.
        let args_match = tx_request.args_hash == call_stack_item.public_inputs().args_hash;
        assert(args_match, "noir function args passed to tx_request must match args in the call_stack_item");
    }

    fn validate_inputs(self) {
        let call_stack_item = self.private_call.call_stack_item;
        let function_data = call_stack_item.function_data();

        let public_inputs = call_stack_item.public_inputs();

        /* TODO(Mike/David): If we are going to have 3 initial calls on the private call stack,
        * then do we still need the `private_call_stack`
        * despite no longer needing a full `previous_kernel`
        */

        assert(function_data.is_private, "Cannot execute a non-private function with the private kernel circuit");
        assert(public_inputs.call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(public_inputs.call_context.is_static_call == false, "Users cannot make a static call");
        
        // The below also prevents delegatecall/staticcall in the base case
        assert(public_inputs.call_context.storage_contract_address.eq(call_stack_item.contract_address()), "Storage contract address must be that of the called contract");
    }

    fn update_end_values(self, public_inputs : &mut KernelCircuitPublicInputsBuilder) {
        // We only initialized constants member of public_inputs so far. Therefore, there must not be any
        // new nullifiers or logs as part of public_inputs.
        
        let new_commitments_empty = arrays::is_empty_array(public_inputs.end.new_commitments.storage);
        assert(new_commitments_empty, "public_inputs.end.new_commitments must start as empty in initial kernel iteration");

        let new_nullifiers_empty = arrays::is_empty_array(public_inputs.end.new_nullifiers.storage);
        assert(new_nullifiers_empty, "public_inputs.end.new_nullifiers must start as empty in initial kernel iteration");

        let end_nullified_commitments_empty = arrays::is_empty_array(public_inputs.end.nullified_commitments.storage);
        assert(end_nullified_commitments_empty, "public_inputs.end.nullified_commitments must start as empty in initial kernel iteration");

        let encrypted_logs_hash_empty = arrays::log_hashes_empty(public_inputs.end.encrypted_logs_hash);
        assert(encrypted_logs_hash_empty, "public_inputs.end.encrypted_logs_hash must start as empty in initial kernel iteration");

        let unencrypted_logs_hash_empty = arrays::log_hashes_empty(public_inputs.end.unencrypted_logs_hash);
        assert(unencrypted_logs_hash_empty, "public_inputs.end.unencrypted_logs_hash must start as empty in initial kernel iteration");

        let read_requests_empty = arrays::is_empty_array(public_inputs.end.read_requests.storage);
        assert(read_requests_empty, "public_inputs.end.read_requests must start as empty in initial kernel iteration");

        let pending_read_requests_empty = arrays::is_empty_array(public_inputs.end.pending_read_requests.storage);
        assert(pending_read_requests_empty, "public_inputs.end.pending_read_requests_empty must start as empty in initial kernel iteration");

        assert(public_inputs.end.encrypted_log_preimages_length == 0,"public_inputs.end.encrypted_log_preimages_length must start as 0 in initial kernel iteration");
        assert(public_inputs.end.unencrypted_log_preimages_length == 0,"public_inputs.end.unencrypted_log_preimages_length must start as 0 in initial kernel iteration");

        // Since it's the first iteration, we need to push the the tx hash nullifier into the `new_nullifiers` array
        public_inputs.end.new_nullifiers.push(self.tx_request.hash());
        
        // Push an empty nullified commitment too since each nullifier must
        // be paired with a nonzero (real or "empty") nullified commitment
        public_inputs.end.nullified_commitments.push(constants_gen::EMPTY_NULLIFIED_COMMITMENT);

        // Note that we do not need to nullify the transaction request nonce anymore.
        // Should an account want to additionally use nonces for replay protection or handling cancellations,
        // they will be able to do so in the account contract logic:
        // https://github.com/AztecProtocol/aztec-packages/issues/660
    }

    pub fn native_private_kernel_circuit_initial(self) -> KernelCircuitPublicInputs {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;
        
        self.initialize_end_values(&mut public_inputs);

        self.validate_inputs();

        common::validate_arrays(self.private_call.call_stack_item.public_inputs());

        self.validate_this_private_call_against_tx_request();

        common::validate_call_stack(self.private_call);

        common::validate_read_requests(
            public_inputs.constants.block_data.note_hash_tree_root(),
            self.private_call.call_stack_item.public_inputs().read_requests,
            self.private_call.read_request_membership_witnesses
        );

        // TODO(dbanks12): feels like update_end_values should happen after contract logic
        self.update_end_values(&mut public_inputs);
        common::update_end_values(self.private_call, &mut public_inputs);

        common::contract_logic(self.private_call,
                          &mut public_inputs,
                          self.tx_request.tx_context.contract_deployment_data,
                          self.tx_request.function_data);

        // TODO(David): What previous kernel proof are we talking about, since this is private-kernel-init
        let (is_previous_state_valid, updated_aggregation_object) = verify_previous_kernel_state(public_inputs.end.aggregation_object,self.private_call.proof);
        assert(is_previous_state_valid);

        public_inputs.end.aggregation_object = updated_aggregation_object;

        public_inputs.finish()
    }
}
