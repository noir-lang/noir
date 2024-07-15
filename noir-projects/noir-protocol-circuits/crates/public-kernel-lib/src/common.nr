use dep::types::{
    abis::{
    call_request::CallRequest, public_call_stack_item::PublicCallStackItem,
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputsBuilder,
    public_kernel_data::PublicKernelData, note_hash::NoteHash, nullifier::Nullifier,
    public_call_data::PublicCallData, public_data_read::PublicDataRead,
    public_data_update_request::PublicDataUpdateRequest, log_hash::{ScopedLogHash, LogHash},
    global_variables::GlobalVariables, combined_constant_data::CombinedConstantData
},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    constants::{
    MAX_L2_TO_L1_MSGS_PER_CALL, MAX_NOTE_HASHES_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL
},
    hash::{compute_siloed_note_hash, compute_siloed_nullifier, compute_l2_to_l1_hash},
    utils::{arrays::{array_length, array_to_bounded_vec}}, traits::{is_empty, is_empty_array}
};

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

pub fn validate_public_call_global_variables(public_call: PublicCallData, constants: CombinedConstantData) {
    let public_call_globals = public_call.call_stack_item.public_inputs.global_variables;
    assert(
        public_call_globals == constants.global_variables, "Global variables injected into the public call do not match constants"
    );
}

// Validates constants injected into the public call are correct.
// Note that the previous_kernel.public_inputs.constants returned from the private kernel tail
// will be empty, so in the first run on of this circuit we load them from the first public
// call, following the same pattern as in the private_kernel_init.
// TODO(@spalladino): This can be a security risk since it allows a sequencer to run public
// circuits with empty global variables. This must be patched by having a differentiated init public
// circuit that runs only once, or by having a way to differentiate when we're coming from a private 
// kernel tail vs from another public run.
pub fn initialize_from_or_validate_public_call_variables(
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
    public_inputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    if public_inputs.constants.global_variables.is_empty() {
        let public_call_global_variables = public_call.call_stack_item.public_inputs.global_variables;
        public_inputs.constants.global_variables = public_call_global_variables;
    } else {
        validate_public_call_global_variables(public_call, previous_kernel.public_inputs.constants);
    }
}

pub fn validate_public_call_non_revert(public_call: PublicCallData) {
    assert(public_call.call_stack_item.public_inputs.revert_code == 0, "Public call cannot be reverted");
}

pub fn initialize_end_values(
    previous_kernel: PublicKernelData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let start = previous_kernel.public_inputs.end;
    circuit_outputs.end.note_hashes = array_to_bounded_vec(start.note_hashes);
    circuit_outputs.end.nullifiers = array_to_bounded_vec(start.nullifiers);
    circuit_outputs.end.l2_to_l1_msgs = array_to_bounded_vec(start.l2_to_l1_msgs);
    circuit_outputs.end.public_data_update_requests = array_to_bounded_vec(start.public_data_update_requests);
    circuit_outputs.end.unencrypted_logs_hashes = array_to_bounded_vec(start.unencrypted_logs_hashes);
    circuit_outputs.end.note_encrypted_logs_hashes = array_to_bounded_vec(start.note_encrypted_logs_hashes);
    circuit_outputs.end.encrypted_logs_hashes = array_to_bounded_vec(start.encrypted_logs_hashes);
    circuit_outputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);
}

// Initialises the circuit outputs with the end state of the previous iteration. 
// Includes data that will be checked and cleared in the tail circuit.
pub fn initialize_non_revertible_values(
    previous_kernel: PublicKernelData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    circuit_outputs.constants = previous_kernel.public_inputs.constants;
    circuit_outputs.fee_payer = previous_kernel.public_inputs.fee_payer;

    // Copy gas-used as-is. Gas used in this iteration will be deducted later in update_(non)_revertible_gas_used.
    circuit_outputs.end.gas_used = previous_kernel.public_inputs.end.gas_used;
    circuit_outputs.end_non_revertible.gas_used = previous_kernel.public_inputs.end_non_revertible.gas_used;

    let start_non_revertible = previous_kernel.public_inputs.end_non_revertible;
    circuit_outputs.end_non_revertible.note_hashes = array_to_bounded_vec(start_non_revertible.note_hashes);
    circuit_outputs.end_non_revertible.nullifiers = array_to_bounded_vec(start_non_revertible.nullifiers);
    circuit_outputs.end_non_revertible.l2_to_l1_msgs = array_to_bounded_vec(start_non_revertible.l2_to_l1_msgs);
    circuit_outputs.end_non_revertible.public_data_update_requests = array_to_bounded_vec(start_non_revertible.public_data_update_requests);
    circuit_outputs.end_non_revertible.unencrypted_logs_hashes = array_to_bounded_vec(start_non_revertible.unencrypted_logs_hashes);
    circuit_outputs.end_non_revertible.note_encrypted_logs_hashes = array_to_bounded_vec(start_non_revertible.note_encrypted_logs_hashes);
    circuit_outputs.end_non_revertible.encrypted_logs_hashes = array_to_bounded_vec(start_non_revertible.encrypted_logs_hashes);
    circuit_outputs.end_non_revertible.public_call_stack = array_to_bounded_vec(start_non_revertible.public_call_stack);

    let start = previous_kernel.public_inputs.validation_requests;
    circuit_outputs.validation_requests.max_block_number = start.for_rollup.max_block_number;
    circuit_outputs.validation_requests.nullifier_read_requests = array_to_bounded_vec(start.nullifier_read_requests);
    circuit_outputs.validation_requests.nullifier_non_existent_read_requests = array_to_bounded_vec(start.nullifier_non_existent_read_requests);
    circuit_outputs.validation_requests.public_data_reads = array_to_bounded_vec(start.public_data_reads);
}

fn perform_static_call_checks(public_call: PublicCallData) {
    let public_inputs = public_call.call_stack_item.public_inputs;
    if public_inputs.call_context.is_static_call {
        // No state changes are allowed for static calls:
        let note_hashes_length = array_length(public_inputs.note_hashes);
        assert(note_hashes_length == 0, "note_hashes must be empty for static calls");

        let nullifiers_length = array_length(public_inputs.nullifiers);
        assert(nullifiers_length == 0, "nullifiers must be empty for static calls");

        let update_requests_length = array_length(public_inputs.contract_storage_update_requests);
        assert(
            update_requests_length == 0, "No contract storage update requests are allowed for static calls"
        );

        let l2_to_l1_msgs_length = array_length(public_inputs.l2_to_l1_msgs);
        assert(l2_to_l1_msgs_length == 0, "l2_to_l1_msgs must be empty for static calls");

        let new_unencrypted_logs_length = array_length(public_inputs.unencrypted_logs_hashes);
        assert(new_unencrypted_logs_length == 0, "No unencrypted logs are allowed for static calls");
    }
}

fn is_valid_caller(request: CallRequest, public_call: PublicCallData) -> bool {
    let call_context = public_call.call_stack_item.public_inputs.call_context;

    let valid_caller_context = request.caller_context.msg_sender.eq(call_context.msg_sender)
        & request.caller_context.storage_contract_address.eq(call_context.storage_contract_address);

    request.caller_contract_address.eq(public_call.call_stack_item.contract_address)
        & (request.caller_context.is_empty() | valid_caller_context)
}

pub fn update_end_non_revertible_call_stack(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let requests = validate_public_call_stack(public_call);
    circuit_outputs.end_non_revertible.public_call_stack.extend_from_bounded_vec(requests);
}

pub fn update_end_call_stack(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let requests = validate_public_call_stack(public_call);
    circuit_outputs.end.public_call_stack.extend_from_bounded_vec(requests);
}

pub fn update_teardown_call_stack(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let requests = validate_public_call_stack(public_call);
    circuit_outputs.public_teardown_call_stack.extend_from_bounded_vec(requests);
}

fn validate_public_call_stack(public_call: PublicCallData) -> BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> {
    let public_call_requests = array_to_bounded_vec(public_call.public_call_stack);
    let hashes = public_call.call_stack_item.public_inputs.public_call_stack_hashes;
    validate_call_requests(public_call_requests, hashes, public_call);
    public_call_requests
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

pub fn update_revertible_gas_used(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let tx_gas_limits = circuit_outputs.constants.tx_context.gas_settings.gas_limits;
    let call_gas_left = public_call.call_stack_item.public_inputs.end_gas_left;
    let accum_end_non_revertible_gas_used = circuit_outputs.end_non_revertible.gas_used;

    // dep::types::debug_log::debug_log_format(
    //     "Updating revertible gas: limit.da={0} limit.l1={1} limit.l2={2} left.da={3} left.l1={4} left.l2={5} used.da={6} used.l1={7} used.l2={8}",
    //     [
    //     tx_gas_limits.da_gas as Field,
    //     tx_gas_limits.l2_gas as Field,
    //     call_gas_left.da_gas as Field,
    //     call_gas_left.l2_gas as Field,
    //     accum_end_non_revertible_gas_used.da_gas as Field,
    //     accum_end_non_revertible_gas_used.l2_gas as Field
    // ]
    // );

    // println(
    //     f"Updating revertible gas: tx_gas_limits={tx_gas_limits} call_gas_left={call_gas_left} accum_end_non_revertible_gas_used={accum_end_non_revertible_gas_used}"
    // );

    circuit_outputs.end.gas_used = tx_gas_limits
        .sub(call_gas_left)
        .sub(accum_end_non_revertible_gas_used);
}

pub fn update_non_revertible_gas_used(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let tx_gas_limits = circuit_outputs.constants.tx_context.gas_settings.gas_limits;
    let call_gas_left = public_call.call_stack_item.public_inputs.end_gas_left;
    let accum_end_gas_used = circuit_outputs.end.gas_used;

    // dep::types::debug_log::debug_log_format(
    //     "Updating non-revertible gas: limit.da={0} limit.l2={1} left.da={2} left.l2={3} used.da={4} used.l2={5}",
    //     [
    //     tx_gas_limits.da_gas as Field,
    //     tx_gas_limits.l2_gas as Field,
    //     call_gas_left.da_gas as Field,
    //     call_gas_left.l2_gas as Field,
    //     accum_end_gas_used.da_gas as Field,
    //     accum_end_gas_used.l2_gas as Field
    // ]
    // );

    circuit_outputs.end_non_revertible.gas_used = tx_gas_limits
        .sub(call_gas_left)
        .sub(accum_end_gas_used);
}

// Validates that the start gas injected into the app circuit matches the remaining gas
pub fn validate_start_gas(public_call: PublicCallData, previous_kernel: PublicKernelData) {
    // If this is a nested call (not an execution request), the start gas is correct as long as the
    // call being processed by this kernel iteration matches the call at the top of the callstack
    // as per the previous kernel's outputs.
    // An execution request's start gas is the remaining gas left in the transaction after the previous kernel.
    // A nested call's start gas is the gas allocated to it by its caller and placed in the callstack.
    if (public_call.call_stack_item.is_execution_request) {
        let public_call_start_gas = public_call.call_stack_item.public_inputs.start_gas_left;
        let tx_gas_limits = previous_kernel.public_inputs.constants.tx_context.gas_settings.gas_limits;
        let computed_start_gas = tx_gas_limits.sub(previous_kernel.public_inputs.end.gas_used).sub(previous_kernel.public_inputs.end_non_revertible.gas_used);
        assert(
            public_call_start_gas == computed_start_gas, "Start gas for public phase does not match transaction gas left"
        );
    }
}

// Validates the transaction fee injected into the app circuit is zero for non-teardown phases
pub fn validate_transaction_fee_is_zero(public_call: PublicCallData) {
    let transaction_fee = public_call.call_stack_item.public_inputs.transaction_fee;
    assert(transaction_fee == 0, "Transaction fee must be zero on setup and app phases");
}

pub fn update_public_end_non_revertible_values(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    // Updates the circuit outputs with new state changes

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    propagate_nullifiers_non_revertible(public_call, circuit_outputs);
    propagate_note_hashes_non_revertible(public_call, circuit_outputs);
    propagate_new_l2_to_l1_messages_non_revertible(public_call, circuit_outputs);
    propagate_new_unencrypted_logs_non_revertible(public_call, circuit_outputs);
    propagate_valid_non_revertible_public_data_update_requests(public_call, circuit_outputs);
}

pub fn update_public_end_values(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // Updates the circuit outputs with new state changes

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(public_call);

    propagate_nullifiers(public_call, circuit_outputs);
    propagate_note_hashes(public_call, circuit_outputs);
    propagate_new_l2_to_l1_messages(public_call, circuit_outputs);
    propagate_new_unencrypted_logs(public_call, circuit_outputs);
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
            circuit_outputs.validation_requests.nullifier_read_requests.push(request.scope(storage_contract_address));
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
            circuit_outputs.validation_requests.nullifier_non_existent_read_requests.push(request.scope(storage_contract_address));
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
            let public_data_update_request = PublicDataUpdateRequest::from_contract_storage_update_request(contract_address, update_request);
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
            let public_data_update_request = PublicDataUpdateRequest::from_contract_storage_update_request(contract_address, update_request);

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
            let public_data_read = PublicDataRead::from_contract_storage_read(contract_address, read_request);
            public_data_reads.push(public_data_read);
        }
    }
    circuit_outputs.validation_requests.public_data_reads.extend_from_bounded_vec(public_data_reads);
}

fn propagate_note_hashes_non_revertible(
    public_call: PublicCallData,
    circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let note_hashes = public_call.call_stack_item.public_inputs.note_hashes;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_note_hashes : BoundedVec<NoteHash, MAX_NOTE_HASHES_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NOTE_HASHES_PER_CALL {
        let new_note_hash = note_hashes[i].value;
        if new_note_hash != 0 {
            let siloed_new_note_hash = compute_siloed_note_hash(storage_contract_address, new_note_hash);
            siloed_note_hashes.push(NoteHash { value: siloed_new_note_hash, counter: note_hashes[i].counter });
        }
    }
    circuit_outputs.end_non_revertible.note_hashes.extend_from_bounded_vec(siloed_note_hashes);
}

fn propagate_note_hashes(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;

    let note_hashes = public_call.call_stack_item.public_inputs.note_hashes;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let mut siloed_note_hashes : BoundedVec<NoteHash, MAX_NOTE_HASHES_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NOTE_HASHES_PER_CALL {
        let new_note_hash = note_hashes[i].value;
        if new_note_hash != 0 {
            let siloed_new_note_hash = compute_siloed_note_hash(storage_contract_address, new_note_hash);
            siloed_note_hashes.push(NoteHash { value: siloed_new_note_hash, counter: note_hashes[i].counter });
        }
    }
    circuit_outputs.end.note_hashes.extend_from_bounded_vec(siloed_note_hashes);
}

fn propagate_nullifiers_non_revertible(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let nullifiers = public_call.call_stack_item.public_inputs.nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_nullifiers : BoundedVec<Nullifier, MAX_NULLIFIERS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NULLIFIERS_PER_CALL {
        let new_nullifier = nullifiers[i].value;
        if new_nullifier != 0 {
            let siloed_new_nullifier = compute_siloed_nullifier(storage_contract_address, new_nullifier);
            siloed_nullifiers.push(
                Nullifier { value: siloed_new_nullifier, counter: nullifiers[i].counter, note_hash: nullifiers[i].note_hash }
            );
        }
    }

    circuit_outputs.end_non_revertible.nullifiers.extend_from_bounded_vec(siloed_nullifiers);
}

fn propagate_nullifiers(public_call: PublicCallData, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    let nullifiers = public_call.call_stack_item.public_inputs.nullifiers;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    let mut siloed_nullifiers : BoundedVec<Nullifier, MAX_NULLIFIERS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NULLIFIERS_PER_CALL {
        let new_nullifier = nullifiers[i].value;
        if new_nullifier != 0 {
            let siloed_new_nullifier = compute_siloed_nullifier(storage_contract_address, new_nullifier);
            siloed_nullifiers.push(
                Nullifier { value: siloed_new_nullifier, counter: nullifiers[i].counter, note_hash: nullifiers[i].note_hash }
            );
        }
    }

    circuit_outputs.end.nullifiers.extend_from_bounded_vec(siloed_nullifiers);
}

fn propagate_new_l2_to_l1_messages_non_revertible(
    public_call: PublicCallData,
    public_inputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    // new l2 to l1 messages
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let l2_to_l1_msgs = public_call_public_inputs.l2_to_l1_msgs;
    let mut l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_L2_TO_L1_MSGS_PER_CALL {
        let msg = l2_to_l1_msgs[i];
        if !is_empty(msg) {
            let l2_to_l1_msgs = compute_l2_to_l1_hash(
                storage_contract_address,
                msg.recipient,
                msg.content,
                public_inputs.constants.tx_context.version,
                public_inputs.constants.tx_context.chain_id
            );
            l2_to_l1_msgs_to_insert.push(l2_to_l1_msgs)
        }
    }
    public_inputs.end_non_revertible.l2_to_l1_msgs.extend_from_bounded_vec(l2_to_l1_msgs_to_insert);
}

fn propagate_new_l2_to_l1_messages(public_call: PublicCallData, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // new l2 to l1 messages
    let public_call_public_inputs = public_call.call_stack_item.public_inputs;
    let storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    let l2_to_l1_msgs = public_call_public_inputs.l2_to_l1_msgs;
    let mut l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_L2_TO_L1_MSGS_PER_CALL {
        let msg = l2_to_l1_msgs[i];
        if !is_empty(msg) {
            let l2_to_l1_msgs = compute_l2_to_l1_hash(
                storage_contract_address,
                msg.recipient,
                msg.content,
                public_inputs.constants.tx_context.version,
                public_inputs.constants.tx_context.chain_id
            );
            l2_to_l1_msgs_to_insert.push(l2_to_l1_msgs)
        }
    }
    public_inputs.end.l2_to_l1_msgs.extend_from_bounded_vec(l2_to_l1_msgs_to_insert);
}

pub fn propagate_new_unencrypted_logs(public_call: PublicCallData, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
    // new unencrypted logs
    let new_logs = public_call.call_stack_item.public_inputs.unencrypted_logs_hashes;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    let mut new_logs_to_insert : BoundedVec<ScopedLogHash, MAX_UNENCRYPTED_LOGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_UNENCRYPTED_LOGS_PER_CALL {
        let new_log = new_logs[i];
        if new_log.value != 0 {
            new_logs_to_insert.push(ScopedLogHash { log_hash: new_log, contract_address: storage_contract_address });
        }
    }
    public_inputs.end.unencrypted_logs_hashes.extend_from_bounded_vec(new_logs_to_insert);
}

pub fn propagate_new_unencrypted_logs_non_revertible(
    public_call: PublicCallData,
    public_inputs: &mut PublicKernelCircuitPublicInputsBuilder
) {
    // new unencrypted logs
    let new_logs = public_call.call_stack_item.public_inputs.unencrypted_logs_hashes;
    let storage_contract_address = public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    let mut new_logs_to_insert : BoundedVec<ScopedLogHash, MAX_UNENCRYPTED_LOGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_UNENCRYPTED_LOGS_PER_CALL {
        let new_log = new_logs[i];
        if new_log.value != 0 {
            new_logs_to_insert.push(ScopedLogHash { log_hash: new_log, contract_address: storage_contract_address });
        }
    }
    public_inputs.end_non_revertible.unencrypted_logs_hashes.extend_from_bounded_vec(new_logs_to_insert);
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
        request.hash == call_stack_item.get_compressed().hash(), "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack"
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
