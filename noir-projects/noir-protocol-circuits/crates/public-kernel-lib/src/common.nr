use dep::types::{
    abis::{
    call_request::CallRequest, public_call_stack_item::PublicCallStackItem,
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputsBuilder, kernel_data::PublicKernelData,
    public_call_data::PublicCallData, public_data_read::PublicDataRead,
    public_data_update_request::PublicDataUpdateRequest, read_request::ReadRequestContext,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    constants::{
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL
},
    hash::{silo_note_hash, silo_nullifier, compute_l2_to_l1_hash, accumulate_sha256},
    utils::{arrays::{array_length, array_to_bounded_vec}}, traits::{is_empty, is_empty_array}
};
use crate::hash::{compute_public_data_tree_index, compute_public_data_tree_value};

// Validates inputs to the kernel circuit that are common to all invocation scenarios.
pub fn validate_inputs(public_call: PublicCallData) {
    // Validates commons inputs for all type of kernel inputs
    let this_call_stack_item: PublicCallStackItem = public_call.call_stack_item;
    assert(
        !this_call_stack_item.contract_address.eq(AztecAddress::zero()), "Contract address cannot be zero"
    );
    assert(this_call_stack_item.function_data.selector.to_field() != 0, "Function signature cannot be zero");
    assert(
        this_call_stack_item.function_data.is_private == false, "Cannot execute a private function with the public kernel circuit"
    );
    assert(public_call.bytecode_hash != 0, "Bytecode hash cannot be zero");
}

pub fn validate_public_call_non_revert(public_call: PublicCallData) {
    assert(public_call.call_stack_item.public_inputs.revert_code == 0, "Public call cannot be reverted");
}

pub fn initialize_revert_code(
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    if previous_kernel.public_inputs.revert_code != 0 {
        circuit_outputs.revert_code = previous_kernel.public_inputs.revert_code;
    } else if public_call.call_stack_item.public_inputs.revert_code != 0 {
        circuit_outputs.revert_code = public_call.call_stack_item.public_inputs.revert_code;
    }
}

// Initialises the circuit outputs with the end state of the previous iteration. 
// Skips data that will be checked and cleared in the tail circuit.
pub fn initialize_emitted_end_values(
    previous_kernel: PublicKernelData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    circuit_outputs.constants = previous_kernel.public_inputs.constants;

    if circuit_outputs.revert_code == 0 {
        let start = previous_kernel.public_inputs.end;
        circuit_outputs.end.new_note_hashes = array_to_bounded_vec(start.new_note_hashes);
        circuit_outputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);
        circuit_outputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);
        circuit_outputs.end.public_data_update_requests = array_to_bounded_vec(start.public_data_update_requests);
        circuit_outputs.end.unencrypted_logs_hash = start.unencrypted_logs_hash;
        circuit_outputs.end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;
        circuit_outputs.end.encrypted_logs_hash = start.encrypted_logs_hash;
        circuit_outputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    }

    let start_non_revertible = previous_kernel.public_inputs.end_non_revertible;
    circuit_outputs.end_non_revertible.new_note_hashes = array_to_bounded_vec(start_non_revertible.new_note_hashes);
    circuit_outputs.end_non_revertible.new_nullifiers = array_to_bounded_vec(start_non_revertible.new_nullifiers);
    circuit_outputs.end_non_revertible.new_l2_to_l1_msgs = array_to_bounded_vec(start_non_revertible.new_l2_to_l1_msgs);
    circuit_outputs.end_non_revertible.public_data_update_requests = array_to_bounded_vec(start_non_revertible.public_data_update_requests);
    circuit_outputs.end_non_revertible.unencrypted_logs_hash = start_non_revertible.unencrypted_logs_hash;
    circuit_outputs.end_non_revertible.unencrypted_log_preimages_length = start_non_revertible.unencrypted_log_preimages_length;
    circuit_outputs.end_non_revertible.encrypted_logs_hash = start_non_revertible.encrypted_logs_hash;
    circuit_outputs.end_non_revertible.encrypted_log_preimages_length = start_non_revertible.encrypted_log_preimages_length;

    // TODO - should be propagated only in initialize_end_values() and clear them in the tail circuit. The 
    // max_block_number must be propagated to the rollup however as a RollupValidationRequest.
    let start = previous_kernel.public_inputs.validation_requests;
    circuit_outputs.validation_requests.max_block_number = start.for_rollup.max_block_number;
    circuit_outputs.validation_requests.public_data_reads = array_to_bounded_vec(start.public_data_reads);
}

// Initialises the circuit outputs with the end state of the previous iteration. 
// Includes data that will be checked and cleared in the tail circuit.
pub fn initialize_end_values(
    previous_kernel: PublicKernelData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    initialize_emitted_end_values(previous_kernel, circuit_outputs);

    if circuit_outputs.revert_code == 0 {
        let start = previous_kernel.public_inputs.end;
        circuit_outputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);
    }

    let start_non_revertible = previous_kernel.public_inputs.end_non_revertible;
    circuit_outputs.end_non_revertible.public_call_stack = array_to_bounded_vec(start_non_revertible.public_call_stack);

    let start = previous_kernel.public_inputs.validation_requests;
    circuit_outputs.validation_requests.max_block_number = previous_kernel.public_inputs.validation_requests.for_rollup.max_block_number;
    circuit_outputs.validation_requests.nullifier_read_requests = array_to_bounded_vec(start.nullifier_read_requests);
    circuit_outputs.validation_requests.nullifier_non_existent_read_requests = array_to_bounded_vec(start.nullifier_non_existent_read_requests);
}

fn perform_static_call_checks(public_call: PublicCallData) {
    let public_inputs = public_call.call_stack_item.public_inputs;
    if public_inputs.call_context.is_static_call {
        // No state changes are allowed for static calls:
        let new_note_hashes_length = array_length(public_inputs.new_note_hashes);
        assert(new_note_hashes_length == 0, "new_note_hashes must be empty for static calls");

        let new_nullifiers_length = array_length(public_inputs.new_nullifiers);
        assert(new_nullifiers_length == 0, "new_nullifiers must be empty for static calls");

        let update_requests_length = array_length(public_inputs.contract_storage_update_requests);
        assert(
            update_requests_length == 0, "No contract storage update requests are allowed for static calls"
        );

        let new_l2_to_l1_msgs_length = array_length(public_inputs.new_l2_to_l1_msgs);
        assert(new_l2_to_l1_msgs_length == 0, "new_l2_to_l1_msgs must be empty for static calls");

        // TODO: reevaluate when implementing https://github.com/AztecProtocol/aztec-packages/issues/1165
        // This 4 magical number is the minimum size of the buffer, since it has to store the total length of all the serialized logs.
        assert(
            public_inputs.unencrypted_log_preimages_length == 4, "No unencrypted logs are allowed for static calls"
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

pub fn update_validation_requests(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // Note that the public kernel cannot modify the max block number value - it simply forwards it to the rollup
    propagate_nullifier_read_requests(public_call, circuit_outputs);
    propagate_nullifier_non_existent_read_requests(public_call, circuit_outputs);
    propagate_valid_public_data_reads(public_call, circuit_outputs);
}

pub fn update_public_end_non_revertible_values(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    // Updates the circuit outputs with new state changes, call stack etc

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    // Update public call stack.
    let public_call_requests = array_to_bounded_vec(public_call.public_call_stack);
    let hashes = public_call.call_stack_item.public_inputs.public_call_stack_hashes;
    validate_call_requests(public_call_requests, hashes, public_call);
    circuit_outputs.end_non_revertible.public_call_stack.extend_from_bounded_vec(public_call_requests);

    propagate_new_nullifiers_non_revertible(public_call, circuit_outputs);
    propagate_new_note_hashes_non_revertible(public_call, circuit_outputs);
    propagate_new_l2_to_l1_messages(public_call, circuit_outputs);
    propagate_valid_non_revertible_public_data_update_requests(public_call, circuit_outputs);
}

pub fn update_public_end_values(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // Updates the circuit outputs with new state changes, call stack etc

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    // Update public call stack.
    let public_call_requests = array_to_bounded_vec(public_call.public_call_stack);
    let hashes = public_call.call_stack_item.public_inputs.public_call_stack_hashes;
    validate_call_requests(public_call_requests, hashes, public_call);
    circuit_outputs.end.public_call_stack.extend_from_bounded_vec(public_call_requests);

    propagate_new_nullifiers(public_call, circuit_outputs);
    propagate_new_note_hashes(public_call, circuit_outputs);

    propagate_new_l2_to_l1_messages(public_call, circuit_outputs);

    propagate_valid_public_data_update_requests(public_call, circuit_outputs);
}

fn propagate_nullifier_read_requests<T>(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let nullifier_read_requests = public_call_public_inputs.nullifier_read_requests;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_CALL {
        let request = nullifier_read_requests[i];
        if !is_empty(request) {
            circuit_outputs.validation_requests.nullifier_read_requests.push(request.to_context(storage_contract_address));
        }
    }
}

fn propagate_nullifier_non_existent_read_requests<T>(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let nullifier_non_existent_read_requests = public_call_public_inputs.nullifier_non_existent_read_requests;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    for i in 0..MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL {
        let request = nullifier_non_existent_read_requests[i];
        if !is_empty(request) {
            circuit_outputs.validation_requests.nullifier_non_existent_read_requests.push(request.to_context(storage_contract_address));
        }
    }
}

fn propagate_valid_public_data_update_requests(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    let update_requests = public_call.call_stack_item.public_inputs.contract_storage_update_requests;

    let mut public_data_update_requests : BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> = BoundedVec::new();

    for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
        let update_request = update_requests[i];
        if (!update_request.is_empty()) {
            let public_data_update_request = PublicDataUpdateRequest {
                leaf_slot: compute_public_data_tree_index(contract_address, update_request.storage_slot),
                new_value: compute_public_data_tree_value(update_request.new_value)
            };

            public_data_update_requests.push(public_data_update_request);
        }
    }
    circuit_outputs.end.public_data_update_requests.extend_from_bounded_vec(public_data_update_requests);
}

fn propagate_valid_non_revertible_public_data_update_requests(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let contract_address = public_call.call_stack_item.contract_address;
    let update_requests = public_call.call_stack_item.public_inputs.contract_storage_update_requests;

    let mut public_data_update_requests : BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> = BoundedVec::new();

    for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
        let update_request = update_requests[i];
        if (!update_request.is_empty()) {
            let public_data_update_request = PublicDataUpdateRequest {
                leaf_slot: compute_public_data_tree_index(contract_address, update_request.storage_slot),
                new_value: compute_public_data_tree_value(update_request.new_value)
            };

            public_data_update_requests.push(public_data_update_request);
        }
    }
    circuit_outputs.end_non_revertible.public_data_update_requests.extend_from_bounded_vec(public_data_update_requests);
}

fn propagate_valid_public_data_reads(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    let read_requests = public_call.call_stack_item.public_inputs.contract_storage_reads;

    let mut public_data_reads : BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_CALL> = BoundedVec::new();

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
    circuit_outputs.validation_requests.public_data_reads.extend_from_bounded_vec(public_data_reads);
}

fn propagate_new_note_hashes_non_revertible(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let new_note_hashes = public_call.call_stack_item.public_inputs.new_note_hashes;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_new_note_hashes : BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_NOTE_HASHES_PER_CALL {
        let new_note_hash = new_note_hashes[i].value;
        if new_note_hash != 0 {
            let siloed_new_note_hash = silo_note_hash(storage_contract_address, new_note_hash);
            siloed_new_note_hashes.push(SideEffect { value: siloed_new_note_hash, counter: new_note_hashes[i].counter });
        }
    }
    circuit_outputs.end_non_revertible.new_note_hashes.extend_from_bounded_vec(siloed_new_note_hashes);
}

fn propagate_new_note_hashes(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let new_note_hashes = public_call.call_stack_item.public_inputs.new_note_hashes;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_new_note_hashes : BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_NOTE_HASHES_PER_CALL {
        let new_note_hash = new_note_hashes[i].value;
        if new_note_hash != 0 {
            let siloed_new_note_hash = silo_note_hash(storage_contract_address, new_note_hash);
            siloed_new_note_hashes.push(SideEffect { value: siloed_new_note_hash, counter: new_note_hashes[i].counter });
        }
    }
    circuit_outputs.end.new_note_hashes.extend_from_bounded_vec(siloed_new_note_hashes);
}

fn propagate_new_nullifiers_non_revertible(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let new_nullifiers = public_call.call_stack_item.public_inputs.new_nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_new_nullifiers : BoundedVec<SideEffectLinkedToNoteHash,MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new();
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

    circuit_outputs.end_non_revertible.new_nullifiers.extend_from_bounded_vec(siloed_new_nullifiers);
}

fn propagate_new_nullifiers(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let new_nullifiers = public_call.call_stack_item.public_inputs.new_nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_new_nullifiers : BoundedVec<SideEffectLinkedToNoteHash,MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new();
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

    circuit_outputs.end.new_nullifiers.extend_from_bounded_vec(siloed_new_nullifiers);
}

fn propagate_new_l2_to_l1_messages(public_call: PublicCallData, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // new l2 to l1 messages
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let new_l2_to_l1_msgs = public_call_public_inputs.new_l2_to_l1_msgs;
    let mut new_l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_CALL {
        let msg = new_l2_to_l1_msgs[i];
        if !is_empty(msg) {
            let new_l2_to_l1_msgs = compute_l2_to_l1_hash(
                storage_contract_address,
                public_inputs.constants.tx_context.version,
                public_inputs.constants.tx_context.chain_id,
                msg
            );
            new_l2_to_l1_msgs_to_insert.push(new_l2_to_l1_msgs)
        }
    }
    public_inputs.end.new_l2_to_l1_msgs.extend_from_bounded_vec(new_l2_to_l1_msgs_to_insert);
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
    previous_unencrypted_logs_hash: Field,
    previous_unencrypted_log_preimages_length: Field,
    public_inputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let current_unencrypted_logs_hash = public_call_public_inputs.unencrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = accumulate_sha256([
        previous_unencrypted_logs_hash,
        current_unencrypted_logs_hash,
    ]);

    // Add log preimages lengths from current iteration to accumulated lengths
    let current_unencrypted_log_preimages_length = public_call_public_inputs.unencrypted_log_preimages_length;
    public_inputs.end.unencrypted_log_preimages_length =
        previous_unencrypted_log_preimages_length + current_unencrypted_log_preimages_length;
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
