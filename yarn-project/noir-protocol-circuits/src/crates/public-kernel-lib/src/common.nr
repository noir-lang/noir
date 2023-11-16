use dep::types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs;
use dep::types::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputsBuilder;
use dep::types::utils::arrays;
use dep::types::abis::previous_kernel_data::PreviousKernelData;
use dep::types::abis::combined_accumulated_data::{CombinedAccumulatedData, CombinedAccumulatedDataBuilder};
use crate::hash::{compute_public_data_tree_index, compute_public_data_tree_value};
use dep::types::hash::{silo_commitment, silo_nullifier, compute_l2_to_l1_hash, accumulate_sha256};
use dep::types::address::Address;
use dep::types::abis::new_contract_data::NewContractData;
use dep::types::abis::complete_address::CompleteAddress;
use dep::types::utils::bounded_vec::BoundedVec;
use crate::abis::public_call_data::PublicCallData;
use dep::types::abis::call_stack_item::PublicCallStackItem;
use dep::types::abis::public_data_update_request::PublicDataUpdateRequest;
use dep::types::contrakt::storage_read::StorageRead;
use dep::types::abis::public_data_read::PublicDataRead;
use dep::types::utils::uint128::U128;

use dep::aztec::constants_gen::{
    MAX_READ_REQUESTS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    EMPTY_NULLIFIED_COMMITMENT,
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_CALL,
};

// Validate that all pre-images on the call stack hash to equal the accumulated data
pub fn validate_call_stack(public_call : PublicCallData) {
    let stack = public_call.call_stack_item.public_inputs.public_call_stack;
    let preimages = public_call.public_call_stack_preimages;

    let our_contract_address = public_call.call_stack_item.contract_address;
    let our_storage_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    let our_msg_sender = public_call.call_stack_item.public_inputs.call_context.msg_sender;
    let our_portal_contract_address =
        public_call.call_stack_item.public_inputs.call_context.portal_contract_address;
    
    assert(stack.len() == preimages.len()); // This is not in the cpp code.
    for i in 0..stack.len() {
        let hash = stack[i];
        let preimage = preimages[i];

        let is_delegate_call = preimage.public_inputs.call_context.is_delegate_call;
        let is_static_call = preimage.public_inputs.call_context.is_static_call;
        let contract_being_called = preimage.contract_address;

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".

        let calculated_hash = if hash == 0 {
            0
        } else {
            preimage.hash()
        };
        if calculated_hash != 0 {
            assert(hash == calculated_hash, "call stack item hash doesn't match preimage hash");
            // here we validate the msg sender for each call on the stack
            // here we validate the storage address for each call on the stack
            // we need to consider regular vs delegate calls for both
            let preimage_msg_sender = preimage.public_inputs.call_context.msg_sender;
            let preimage_storage_address = preimage.public_inputs.call_context.storage_contract_address;

            if is_delegate_call {
                assert(preimage_msg_sender.eq(our_msg_sender), "call_stack_msg_sender doesn't match expected_msg_sender");
                assert(preimage_storage_address.eq(our_storage_address), "call_stack_storage_address doesnt match expected_storage_address");
                // if it is a delegate call then we check that the portal contract in the pre image is our portal contract
                let preimage_portal_address = preimage.public_inputs.call_context.portal_contract_address;
                let expected_portal_address = our_portal_contract_address;
                assert(preimage_portal_address.eq(expected_portal_address), "call_stack_portal_contract_address doesnt match expected_portal_address");
            } else {
                assert(preimage_msg_sender.eq(our_contract_address), "call_stack_msg_sender doesn't match expected_msg_sender");
                assert(preimage_storage_address.eq(contract_being_called), "call_stack_storage_address doesnt match expected_storage_address");
            }


            let num_contract_storage_update_requests = preimage.public_inputs.contract_storage_update_requests.len();
            if is_static_call {
                assert(num_contract_storage_update_requests == 0,"contract_storage_update_requests should be empty for static call");
            }
        }
    }
}

// Validates the call context of the current iteration
pub fn validate_call_context(public_call: PublicCallData){
    let call_stack_item = public_call.call_stack_item;
    let is_delegate_call = call_stack_item.public_inputs.call_context.is_delegate_call;
    let is_static_call = call_stack_item.public_inputs.call_context.is_static_call;
    let contract_address = call_stack_item.contract_address;
    let storage_contract_address = call_stack_item.public_inputs.call_context.storage_contract_address;
    let contract_storage_update_requests_length =
        call_stack_item.public_inputs.contract_storage_update_requests.len();

    if is_delegate_call{
    assert(!contract_address.eq(storage_contract_address),
                      "curent contract address must not match storage contract address for delegate calls");
    }
    if is_static_call {

    assert(contract_storage_update_requests_length == 0,
        "No contract storage update requests are allowed for static calls");
    }
}

// Validates the kernel execution of the current iteration
pub fn validate_kernel_execution(public_call: PublicCallData) {
    validate_call_context(public_call);
    validate_call_stack(public_call);
}

//Validates inputs to the kernel circuit that are common to all invocation scenarios
pub fn validate_inputs(public_call: PublicCallData){
    // Validates commons inputs for all type of kernel inputs
    let this_call_stack_item: PublicCallStackItem = public_call.call_stack_item;
    assert(this_call_stack_item.public_inputs.call_context.is_contract_deployment == false,
                      "Contract deployment can't be a public function");
    assert(!this_call_stack_item.contract_address.eq(Address::ZERO()),
                      "Contract address must be non-zero");
    assert(this_call_stack_item.function_data.selector.to_field() != 0,
                      "Function signature must be non-zero");
    assert(this_call_stack_item.function_data.is_constructor == false,
                      "Constructors can't be public functions");
    assert(this_call_stack_item.function_data.is_private == false,
                      "Cannot execute a private function with the public kernel circuit");
    assert(public_call.bytecode_hash != 0,
                      "Bytecode hash must be non-zero");

    if (this_call_stack_item.function_data.is_internal) {
        let target = this_call_stack_item.contract_address;
        let sender = this_call_stack_item.public_inputs.call_context.msg_sender;

        assert(target.eq(sender), "call is internal, but msg_sender is not self");
    }
}

pub fn perform_static_call_checks(public_call: PublicCallData){
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let is_static_call = public_call_public_inputs.call_context.is_static_call;
    // If the call is a static call, there should be no new commitments or nullifiers.
    let new_commitments = public_call_public_inputs.new_commitments;
    let new_nullifiers = public_call_public_inputs.new_nullifiers;

    if (is_static_call) {
        // No state changes are allowed for static calls:
        assert(arrays::is_empty_array(new_commitments), "new_commitments must be empty for static calls");
        assert(arrays::is_empty_array(new_nullifiers), "new_nullifiers must be empty for static calls");
    }
}

pub fn propagate_valid_public_data_update_requests(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder){
    let contract_address = public_call.call_stack_item.contract_address;
    let update_requests = public_call.call_stack_item.public_inputs.contract_storage_update_requests;

    let mut public_data_update_requests : BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> = BoundedVec::new(PublicDataUpdateRequest::empty());

    assert(update_requests.len() == MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX); // This is not done in the cpp code.

    for  i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        let update_request = update_requests[i];
        if (!update_request.is_empty()) {
            let public_data_update_request = PublicDataUpdateRequest {
                leaf_index: compute_public_data_tree_index(contract_address, update_request.storage_slot),
                old_value: compute_public_data_tree_value(update_request.old_value),
                new_value: compute_public_data_tree_value(update_request.new_value),
            };

            public_data_update_requests.push(public_data_update_request);
        }
    }
    circuit_outputs.end.public_data_update_requests.push_vec(public_data_update_requests);

}


pub fn initialize_end_values(previous_kernel : PreviousKernelData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder) {
    // Initialises the circuit outputs with the end state of the previous iteration
    circuit_outputs.constants = previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    let start = previous_kernel.public_inputs.end;

    circuit_outputs.end.new_commitments = arrays::array_to_bounded_vec(start.new_commitments, arrays::is_empty, 0);
    circuit_outputs.end.new_nullifiers = arrays::array_to_bounded_vec(start.new_nullifiers, arrays::is_empty, 0);

    circuit_outputs.end.private_call_stack = arrays::array_to_bounded_vec(start.private_call_stack, arrays::is_empty, 0);
    circuit_outputs.end.public_call_stack = arrays::array_to_bounded_vec(start.public_call_stack, arrays::is_empty, 0);
    circuit_outputs.end.new_l2_to_l1_msgs = arrays::array_to_bounded_vec(start.new_l2_to_l1_msgs, arrays::is_empty, 0);

    circuit_outputs.end.optionally_revealed_data = start.optionally_revealed_data;

    circuit_outputs.end.public_data_update_requests = arrays::array_to_bounded_vec(start.public_data_update_requests, |pdu: PublicDataUpdateRequest| pdu.is_empty(), PublicDataUpdateRequest::empty());
    circuit_outputs.end.public_data_reads = arrays::array_to_bounded_vec(start.public_data_reads, |pdr: PublicDataRead| pdr.is_empty(), PublicDataRead::empty());

    // Public kernel does not modify encrypted logs values --> we just copy them to output
    circuit_outputs.end.encrypted_logs_hash = start.encrypted_logs_hash;
    circuit_outputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;

    circuit_outputs.end.new_contracts = arrays::array_to_bounded_vec(previous_kernel.public_inputs.end.new_contracts, |ncd: NewContractData| ncd.is_empty(), NewContractData::default());
}

pub fn update_public_end_values(public_call: PublicCallData, circuit_outputs : &mut KernelCircuitPublicInputsBuilder) {
    // Updates the circuit outputs with new state changes, call stack etc
    circuit_outputs.is_private = false;

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let public_call_stack = arrays::array_to_bounded_vec(public_call_public_inputs.public_call_stack, arrays::is_empty, 0);
    circuit_outputs.end.public_call_stack.push_vec(public_call_stack);

    // don't update pending_read_requests, because those just get passed through without any change

    propagate_new_nullifiers(public_call, circuit_outputs);
    propagate_new_commitments(public_call, circuit_outputs);

    propagate_new_l2_to_l1_messages(public_call, circuit_outputs);

    propagate_valid_public_data_update_requests(public_call, circuit_outputs);

    propagate_valid_public_data_reads(public_call, circuit_outputs);
}

pub fn propagate_valid_public_data_reads(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder){
    let contract_address = public_call.call_stack_item.contract_address;
    let read_requests = public_call.call_stack_item.public_inputs.contract_storage_reads;

    let mut public_data_reads : BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_CALL> = BoundedVec::new(PublicDataRead::empty());

    for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
        let read_request: StorageRead = read_requests[i];
        if !read_request.is_empty() {
            let public_data_read = PublicDataRead {
                leaf_index: compute_public_data_tree_index(contract_address, read_request.storage_slot),
                value: compute_public_data_tree_value(read_request.current_value),
            };
            public_data_reads.push(public_data_read);
        }
    }
   circuit_outputs.end.public_data_reads.push_vec(public_data_reads);
}

pub fn propagate_new_commitments(public_call: PublicCallData, circuit_outputs : &mut KernelCircuitPublicInputsBuilder) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let new_commitments = public_call.call_stack_item.public_inputs.new_commitments;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_new_commitments : BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL> = BoundedVec::new(0);
    assert(new_commitments.len() == MAX_NEW_COMMITMENTS_PER_CALL);
    for i in 0..MAX_NEW_COMMITMENTS_PER_CALL {
        let new_commitment = new_commitments[i];
        if new_commitment != 0 {
            siloed_new_commitments.push(silo_commitment(storage_contract_address, new_commitment));
        }
    }
    circuit_outputs.end.new_commitments.push_vec(siloed_new_commitments);
}

pub fn propagate_new_nullifiers(public_call: PublicCallData, circuit_outputs : &mut KernelCircuitPublicInputsBuilder) {
    let new_nullifiers = public_call.call_stack_item.public_inputs.new_nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_new_nullifiers : BoundedVec<Field,MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new(0);
    assert(new_nullifiers.len() == MAX_NEW_NULLIFIERS_PER_CALL);
    for i in 0..MAX_NEW_NULLIFIERS_PER_CALL {
        let new_nullifier = new_nullifiers[i];
        if new_nullifier != 0 {
            siloed_new_nullifiers.push(silo_nullifier(storage_contract_address, new_nullifier));
        }
    }

    circuit_outputs.end.new_nullifiers.push_vec(siloed_new_nullifiers);
}


pub fn propagate_new_l2_to_l1_messages(public_call: PublicCallData, public_inputs:  &mut KernelCircuitPublicInputsBuilder){
    // new l2 to l1 messages
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let portal_contract_address = public_call.portal_contract_address;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let new_l2_to_l1_msgs = public_call_public_inputs.new_l2_to_l1_msgs;
    let mut new_l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_CALL {
        let msg_is_zero = new_l2_to_l1_msgs[i] == 0;
        if msg_is_zero == false {
            let new_l2_to_l1_msgs = compute_l2_to_l1_hash(storage_contract_address,
                                                                           public_inputs.constants.tx_context.version,
                                                                           portal_contract_address,
                                                                           public_inputs.constants.tx_context.chain_id,
                                                                           new_l2_to_l1_msgs[i]);
            new_l2_to_l1_msgs_to_insert.push(new_l2_to_l1_msgs)
        }
    }
    public_inputs.end.new_l2_to_l1_msgs.push_vec(new_l2_to_l1_msgs_to_insert);
}

/**
 * @brief Accumulates unencrypted logs hashes and lengths.
 * @param The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 * @note See the following thread if not clear:
 *       https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426
 * @note Used by public kernels which had previous iterations.
 */
pub fn accumulate_unencrypted_logs(public_call: PublicCallData,
                                   previous_kernel: PreviousKernelData,
                                   public_inputs:  &mut KernelCircuitPublicInputsBuilder)
{
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let previous_kernel_end = previous_kernel.public_inputs.end;
    let previous_unencrypted_logs_hash = previous_kernel_end.unencrypted_logs_hash;

    let current_unencrypted_logs_hash = public_call_public_inputs.unencrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = accumulate_sha256([
        U128::from_field(previous_unencrypted_logs_hash[0]),            
        U128::from_field(previous_unencrypted_logs_hash[1]),
        U128::from_field(current_unencrypted_logs_hash[0]),
        U128::from_field(current_unencrypted_logs_hash[1])
    ]);

    // Add log preimages lengths from current iteration to accumulated lengths
    let current_unencrypted_log_preimages_length = public_call_public_inputs.unencrypted_log_preimages_length;
    public_inputs.end.unencrypted_log_preimages_length =
        previous_kernel_end.unencrypted_log_preimages_length + current_unencrypted_log_preimages_length;
}


/**
 * @brief Validates that the call stack item for this circuit iteration is at the top of the call stack
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
 pub fn validate_this_public_call_hash(public_call: PublicCallData, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
    // If public call stack is empty, we bail so array_pop doesn't throw_or_abort
    assert(public_inputs.end.public_call_stack.len() != 0 , "Public call stack can't be empty");

    // Pops the current function execution from the stack and validates it against the call stack item

    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    let popped_public_call_hash = public_inputs.end.public_call_stack.pop();
    let calculated_this_public_call_hash = public_call.call_stack_item.hash();

    assert(
        popped_public_call_hash == calculated_this_public_call_hash,
        "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack");
}