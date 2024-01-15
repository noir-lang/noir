use dep::types::{
    abis::{
        call_request::CallRequest,
        call_stack_item::PublicCallStackItem,
        combined_accumulated_data::{CombinedAccumulatedData, CombinedAccumulatedDataBuilder},
        complete_address::CompleteAddress,
        kernel_circuit_public_inputs::KernelCircuitPublicInputsBuilder,
        new_contract_data::NewContractData,
        previous_kernel_data::PreviousKernelData,
        public_call_data::PublicCallData,
        public_circuit_public_inputs::PublicCircuitPublicInputs,
        public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    },
    address::AztecAddress,
    contrakt::{
        storage_read::StorageRead,
        storage_update_request::StorageUpdateRequest,
    },
    constants::{
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        MAX_PUBLIC_DATA_READS_PER_CALL,
    },
    hash::{silo_commitment, silo_nullifier, compute_l2_to_l1_hash, accumulate_sha256},
    utils::{
        arrays::{array_length, array_to_bounded_vec},
        bounded_vec::BoundedVec,
    },
    traits::is_empty_array
};
use crate::hash::{compute_public_data_tree_index, compute_public_data_tree_value};

// Validates inputs to the kernel circuit that are common to all invocation scenarios.
pub fn validate_inputs(public_call: PublicCallData) {
    // Validates commons inputs for all type of kernel inputs
    let this_call_stack_item: PublicCallStackItem = public_call.call_stack_item;
    assert(
        this_call_stack_item.public_inputs.call_context.is_contract_deployment == false, "Contract deployment cannot be a public function"
    );
    assert(
        !this_call_stack_item.contract_address.eq(AztecAddress::zero()), "Contract address cannot be zero"
    );
    assert(this_call_stack_item.function_data.selector.to_field() != 0, "Function signature cannot be zero");
    assert(
        this_call_stack_item.function_data.is_constructor == false, "Constructors cannot be public functions"
    );
    assert(
        this_call_stack_item.function_data.is_private == false, "Cannot execute a private function with the public kernel circuit"
    );
    assert(public_call.bytecode_hash != 0, "Bytecode hash cannot be zero");
}

pub fn initialize_end_values(
    previous_kernel: PreviousKernelData,
    circuit_outputs: &mut KernelCircuitPublicInputsBuilder
) {
    // Initialises the circuit outputs with the end state of the previous iteration
    circuit_outputs.constants = previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    let start = previous_kernel.public_inputs.end;

    circuit_outputs.end.new_commitments = array_to_bounded_vec(start.new_commitments);
    circuit_outputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);

    circuit_outputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);
    circuit_outputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);
    circuit_outputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);

    circuit_outputs.end.optionally_revealed_data = start.optionally_revealed_data;

    circuit_outputs.end.public_data_update_requests = array_to_bounded_vec(start.public_data_update_requests);
    circuit_outputs.end.public_data_reads = array_to_bounded_vec(start.public_data_reads);

    // Public kernel does not modify encrypted logs values --> we just copy them to output
    circuit_outputs.end.encrypted_logs_hash = start.encrypted_logs_hash;
    circuit_outputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;

    circuit_outputs.end.new_contracts = array_to_bounded_vec(previous_kernel.public_inputs.end.new_contracts);
}

fn perform_static_call_checks(public_call: PublicCallData) {
    let public_inputs = public_call.call_stack_item.public_inputs;
    if public_inputs.call_context.is_static_call {
        // No state changes are allowed for static calls:
        let new_commitments_length = array_length(public_inputs.new_commitments);
        assert(new_commitments_length == 0, "new_commitments must be empty for static calls");

        let new_nullifiers_length = array_length(public_inputs.new_nullifiers);
        assert(new_nullifiers_length == 0, "new_nullifiers must be empty for static calls");

        let update_requests_length = array_length(public_inputs.contract_storage_update_requests);
        assert(
            update_requests_length == 0, "No contract storage update requests are allowed for static calls"
        );
    }
}

fn is_valid_caller(request: CallRequest, public_call: PublicCallData) -> bool {
    let call_context = public_call.call_stack_item.public_inputs.call_context;
    let valid_caller_context = request.caller_context.msg_sender.eq(call_context.msg_sender)
        & request.caller_context.storage_contract_address.eq(call_context.storage_contract_address);
    request.caller_contract_address.eq(public_call.call_stack_item.contract_address)
        & (request.caller_context.is_empty() | valid_caller_context)
}

fn validate_call_requests<N>(
    call_requests: BoundedVec<CallRequest, N>,
    hashes: [Field; N],
    public_call: PublicCallData
) {
    assert_eq(
        array_length(hashes), call_requests.len(), "call requests length does not match the expected length"
    );
    for i in 0..N {
        let hash = hashes[i];
        if hash != 0 {
            let request = call_requests.get_unchecked(i);
            assert_eq(request.hash, hash, "call stack hash does not match call request hash");
            assert(is_valid_caller(request, public_call), "invalid caller");
        }
    }
}

pub fn update_public_end_values(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder) {
    // Updates the circuit outputs with new state changes, call stack etc
    circuit_outputs.is_private = false;

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    // Update public call stack.
    let public_call_requests = array_to_bounded_vec(public_call.public_call_stack);
    let hashes = public_call.call_stack_item.public_inputs.public_call_stack_hashes;
    validate_call_requests(public_call_requests, hashes, public_call);
    circuit_outputs.end.public_call_stack.push_vec(public_call_requests);

    propagate_new_nullifiers(public_call, circuit_outputs);
    propagate_new_commitments(public_call, circuit_outputs);

    propagate_new_l2_to_l1_messages(public_call, circuit_outputs);

    propagate_valid_public_data_update_requests(public_call, circuit_outputs);

    propagate_valid_public_data_reads(public_call, circuit_outputs);
}

fn propagate_valid_public_data_update_requests(
    public_call: PublicCallData,
    circuit_outputs: &mut KernelCircuitPublicInputsBuilder
) {
    let contract_address = public_call.call_stack_item.contract_address;
    let update_requests = public_call.call_stack_item.public_inputs.contract_storage_update_requests;

    let mut public_data_update_requests : BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> = BoundedVec::new(PublicDataUpdateRequest::empty());

    for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        let update_request = update_requests[i];
        if (!update_request.is_empty()) {
            let public_data_update_request = PublicDataUpdateRequest {
                leaf_slot: compute_public_data_tree_index(contract_address, update_request.storage_slot),
                old_value: compute_public_data_tree_value(update_request.old_value),
                new_value: compute_public_data_tree_value(update_request.new_value)
            };

            public_data_update_requests.push(public_data_update_request);
        }
    }
    circuit_outputs.end.public_data_update_requests.push_vec(public_data_update_requests);
}

fn propagate_valid_public_data_reads(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder) {
    let contract_address = public_call.call_stack_item.contract_address;
    let read_requests = public_call.call_stack_item.public_inputs.contract_storage_reads;

    let mut public_data_reads : BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_CALL> = BoundedVec::new(PublicDataRead::empty());

    for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
        let read_request: StorageRead = read_requests[i];
        if !read_request.is_empty() {
            let public_data_read = PublicDataRead {
                leaf_slot: compute_public_data_tree_index(contract_address, read_request.storage_slot),
                value: compute_public_data_tree_value(read_request.current_value)
            };
            public_data_reads.push(public_data_read);
        }
    }
    circuit_outputs.end.public_data_reads.push_vec(public_data_reads);
}

fn propagate_new_commitments(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let new_commitments = public_call.call_stack_item.public_inputs.new_commitments;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_new_commitments : BoundedVec<SideEffect, MAX_NEW_COMMITMENTS_PER_CALL> = BoundedVec::new(SideEffect::empty());
    for i in 0..MAX_NEW_COMMITMENTS_PER_CALL {
        let new_commitment = new_commitments[i].value;
        if new_commitment != 0 {
            let siloed_new_commitment = silo_commitment(storage_contract_address, new_commitment);
            siloed_new_commitments.push(SideEffect { value: siloed_new_commitment, counter: new_commitments[i].counter });
        }
    }
    circuit_outputs.end.new_commitments.push_vec(siloed_new_commitments);
}

fn propagate_new_nullifiers(public_call: PublicCallData, circuit_outputs: &mut KernelCircuitPublicInputsBuilder) {
    let new_nullifiers = public_call.call_stack_item.public_inputs.new_nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_new_nullifiers : BoundedVec<SideEffectLinkedToNoteHash,MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new(SideEffectLinkedToNoteHash::empty());
    for i in 0..MAX_NEW_NULLIFIERS_PER_CALL {
        let new_nullifier = new_nullifiers[i].value;
        if new_nullifier != 0 {
            let siloed_new_nullifier = silo_nullifier(storage_contract_address, new_nullifier);
            siloed_new_nullifiers.push(
                SideEffectLinkedToNoteHash {
                value: siloed_new_nullifier,
                counter: new_nullifiers[i].counter,
                note_hash: new_nullifiers[i].note_hash
            }
            );
        }
    }

    circuit_outputs.end.new_nullifiers.push_vec(siloed_new_nullifiers);
}

fn propagate_new_l2_to_l1_messages(public_call: PublicCallData, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
    // new l2 to l1 messages
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let portal_contract_address = public_call.portal_contract_address;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let new_l2_to_l1_msgs = public_call_public_inputs.new_l2_to_l1_msgs;
    let mut new_l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_CALL {
        let msg_is_zero = new_l2_to_l1_msgs[i] == 0;
        if msg_is_zero == false {
            let new_l2_to_l1_msgs = compute_l2_to_l1_hash(
                storage_contract_address,
                public_inputs.constants.tx_context.version,
                portal_contract_address,
                public_inputs.constants.tx_context.chain_id,
                new_l2_to_l1_msgs[i]
            );
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
pub fn accumulate_unencrypted_logs(
    public_call: PublicCallData,
    previous_kernel: PreviousKernelData,
    public_inputs: &mut KernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let previous_kernel_end = previous_kernel.public_inputs.end;
    let previous_unencrypted_logs_hash = previous_kernel_end.unencrypted_logs_hash;

    let current_unencrypted_logs_hash = public_call_public_inputs.unencrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = accumulate_sha256([
        U128::from_integer(previous_unencrypted_logs_hash[0]),            
        U128::from_integer(previous_unencrypted_logs_hash[1]),
        U128::from_integer(current_unencrypted_logs_hash[0]),
        U128::from_integer(current_unencrypted_logs_hash[1])
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
pub fn validate_call_against_request(public_call: PublicCallData, request: CallRequest) {
    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    let call_stack_item = public_call.call_stack_item;

    assert(
        request.hash == call_stack_item.hash(), "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack"
    );

    let call_context = call_stack_item.public_inputs.call_context;

    if (call_stack_item.function_data.is_internal) {
        assert(
            call_stack_item.contract_address.eq(call_context.msg_sender), "msg_sender must be self for internal calls"
        );
    }

    if call_context.is_delegate_call {
        let caller_context = request.caller_context;
        assert(!caller_context.is_empty(), "caller context cannot be empty for delegate calls");
        assert(
            call_context.msg_sender.eq(caller_context.msg_sender), "call stack msg_sender does not match expected msg_sender for delegate calls"
        );
        assert(
            call_context.storage_contract_address.eq(caller_context.storage_contract_address), "call stack storage address does not match expected contract address for delegate calls"
        );
        assert(
            !call_stack_item.contract_address.eq(call_context.storage_contract_address), "curent contract address must not match storage contract address for delegate calls"
        );
    } else {
        let caller_contract_address = request.caller_contract_address;
        assert(
            call_context.msg_sender.eq(caller_contract_address), "call stack msg_sender does not match caller contract address"
        );
        assert(
            call_context.storage_contract_address.eq(call_stack_item.contract_address), "call stack storage address does not match expected contract address"
        );
    }
}
