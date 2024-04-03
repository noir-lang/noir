use dep::std;
use dep::types::{
    abis::{
    call_request::CallRequest, accumulated_data::PrivateAccumulatedData,
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputsBuilder,
    max_block_number::MaxBlockNumber, membership_witness::NoteHashReadRequestMembershipWitness,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    private_kernel::private_call_data::PrivateCallData, kernel_data::PrivateKernelData,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    address::{AztecAddress, EthAddress, PartialAddress, compute_initialization_hash},
    contract_class_id::ContractClassId,
    constants::{
    MAX_NEW_NULLIFIERS_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL
},
    hash::{
    compute_l2_to_l1_hash, compute_logs_hash, private_functions_root_from_siblings, silo_note_hash,
    silo_nullifier, stdlib_recursion_verification_key_compress_native_vk
},
    merkle_tree::check_membership,
    utils::{arrays::{array_length, array_to_bounded_vec, validate_array}},
    traits::{is_empty, is_empty_array}
};

pub fn validate_arrays(app_public_inputs: PrivateCircuitPublicInputs) {
    // Each of the following arrays is expected to be zero-padded.
    // In addition, some of the following arrays (new_note_hashes, etc...) are passed
    // to extend_from_array_to_array() routines which rely on the passed arrays to be well-formed.

    validate_array(app_public_inputs.return_values);
    validate_array(app_public_inputs.note_hash_read_requests);
    validate_array(app_public_inputs.nullifier_read_requests);
    validate_array(app_public_inputs.nullifier_key_validation_requests);
    validate_array(app_public_inputs.new_note_hashes);
    validate_array(app_public_inputs.new_nullifiers);
    validate_array(app_public_inputs.private_call_stack_hashes);
    validate_array(app_public_inputs.public_call_stack_hashes);
    validate_array(app_public_inputs.new_l2_to_l1_msgs);
    // encrypted_logs_hash and unencrypted_logs_hash have their own integrity checks.
}

// Validate all read requests against the historical note hash tree root.
// Use their membership witnesses to do so. If the historical root is not yet
// initialized, initialize it using the first read request here (if present).
//
// More info here:
// - https://discourse.aztec.network/t/to-read-or-not-to-read/178
// - https://discourse.aztec.network/t/spending-notes-which-havent-yet-been-inserted/180
pub fn validate_note_hash_read_requests(
    historical_note_hash_tree_root: Field,
    read_requests: [SideEffect; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    read_request_membership_witnesses: [NoteHashReadRequestMembershipWitness; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL]
) {
    // membership witnesses must resolve to the same note hash tree root
    // for every request in all kernel iterations
    for rr_idx in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_CALL {
        let read_request = read_requests[rr_idx].value;
        let witness = read_request_membership_witnesses[rr_idx];

        // A pending note hash is the one that is not yet added to note hash tree
        // A "transient read" is when we try to "read" a pending note hash within a transaction
        // between function calls, as opposed to reading the outputs of a previous transaction
        // which is a "pending read".
        // A transient read is when we try to "read" a pending note hash
        // We determine if it is a transient read depending on the leaf index from the membership witness
        // Note that the Merkle membership proof would be null and void in case of an transient read
        // but we use the leaf index as a placeholder to detect a 'pending note read'.

        if (read_request != 0) & (witness.is_transient == false) {
            assert(
                check_membership(
                    read_request,
                    witness.leaf_index,
                    witness.sibling_path,
                    historical_note_hash_tree_root
                ), "note hash tree root mismatch"
            );
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1354): do we need to enforce
            // that a non-transient read_request was derived from the proper/current contract address?
        }
    }
}

pub fn initialize_end_values(
    previous_kernel: PrivateKernelData,
    public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder
) {
    public_inputs.constants = previous_kernel.public_inputs.constants;
    public_inputs.min_revertible_side_effect_counter = previous_kernel.public_inputs.min_revertible_side_effect_counter;

    let start = previous_kernel.public_inputs.validation_requests;
    public_inputs.validation_requests.max_block_number = start.for_rollup.max_block_number;
    public_inputs.validation_requests.note_hash_read_requests = array_to_bounded_vec(start.note_hash_read_requests);
    public_inputs.validation_requests.nullifier_read_requests = array_to_bounded_vec(start.nullifier_read_requests);
    public_inputs.validation_requests.nullifier_key_validation_requests = array_to_bounded_vec(start.nullifier_key_validation_requests);

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    let start = previous_kernel.public_inputs.end;

    public_inputs.end.new_note_hashes = array_to_bounded_vec(start.new_note_hashes);
    public_inputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);

    public_inputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);
    public_inputs.end.public_call_stack = array_to_bounded_vec(start.public_call_stack);
    public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);

    public_inputs.end.encrypted_logs_hash = start.encrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = start.unencrypted_logs_hash;

    public_inputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    public_inputs.end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;
}

fn perform_static_call_checks(private_call: PrivateCallData) {
    let public_inputs = private_call.call_stack_item.public_inputs;
    let is_static_call = public_inputs.call_context.is_static_call;
    if is_static_call {
        // No state changes are allowed for static calls:
        assert(
            is_empty_array(public_inputs.new_note_hashes), "new_note_hashes must be empty for static calls"
        );
        assert(
            is_empty_array(public_inputs.new_nullifiers), "new_nullifiers must be empty for static calls"
        );

        let new_l2_to_l1_msgs_length = array_length(public_inputs.new_l2_to_l1_msgs);
        assert(new_l2_to_l1_msgs_length == 0, "new_l2_to_l1_msgs must be empty for static calls");

        // TODO: reevaluate when implementing https://github.com/AztecProtocol/aztec-packages/issues/1165
        // This 4 magical number is the minimum size of the buffer, since it has to store the total length of all the serialized logs.
        assert(
            public_inputs.encrypted_log_preimages_length == 4, "No encrypted logs are allowed for static calls"
        );

        assert(
            public_inputs.unencrypted_log_preimages_length == 4, "No unencrypted logs are allowed for static calls"
        );
    }
}

fn is_valid_caller(request_from_stack: CallRequest, fn_being_verified: PrivateCallData) -> bool {
    let call_context = fn_being_verified.call_stack_item.public_inputs.call_context;

    let valid_caller_context = request_from_stack.caller_context.msg_sender.eq(call_context.msg_sender)
        & request_from_stack.caller_context.storage_contract_address.eq(call_context.storage_contract_address);

    request_from_stack.caller_contract_address.eq(fn_being_verified.call_stack_item.contract_address)
        & (request_from_stack.caller_context.is_empty() | valid_caller_context)
}

fn validate_call_requests<N>(
    call_requests: BoundedVec<CallRequest, N>,
    hashes: [Field; N],
    private_call: PrivateCallData
) {
    assert_eq(
        array_length(hashes), call_requests.len(), "call requests length does not match the expected length"
    );
    for i in 0..N {
        let hash = hashes[i];
        if hash != 0 {
            let request = call_requests.get_unchecked(i);
            assert_eq(request.hash, hash, "call stack hash does not match call request hash");
            assert(is_valid_caller(request, private_call), "invalid caller");
        }
    }
}

pub fn update_end_values(
    private_call: PrivateCallData,
    public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder
) {
    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(private_call);

    let private_call_public_inputs = private_call.call_stack_item.public_inputs;

    let storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;

    // Update the max block number if the private call requested a lower one.
    public_inputs.validation_requests.max_block_number = MaxBlockNumber::min(public_inputs.validation_requests.max_block_number, private_call_public_inputs.max_block_number);

    // Transient read requests and witnesses are accumulated in public_inputs.end
    // We silo the read requests (domain separation per contract address)
    let read_requests = private_call_public_inputs.note_hash_read_requests;
    let read_request_membership_witnesses = private_call.note_hash_read_request_membership_witnesses;
    let mut siloed_read_requests: BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_CALL {
        let read_request = read_requests[i].value;
        let witness = read_request_membership_witnesses[i];
        if witness.is_transient & (read_request != 0) { // only forward transient to public inputs
            siloed_read_requests.push(
                SideEffect { counter: read_requests[i].counter, value: silo_note_hash(storage_contract_address, read_request) }
            )
        }
    }
    public_inputs.validation_requests.note_hash_read_requests.extend_from_bounded_vec(siloed_read_requests);

    // Nullifier read request.
    let nullifier_read_requests = private_call_public_inputs.nullifier_read_requests;
    for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_CALL {
        let request = nullifier_read_requests[i];
        if !is_empty(request) {
            public_inputs.validation_requests.nullifier_read_requests.push(request.to_context(storage_contract_address));
        }
    }

    // Nullifier key validation requests.
    let nullifier_key_validation_requests = private_call_public_inputs.nullifier_key_validation_requests;
    for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL {
        let request = nullifier_key_validation_requests[i];
        if !is_empty(request) {
            public_inputs.validation_requests.nullifier_key_validation_requests.push(request.to_context(storage_contract_address));
        }
    }

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    //
    // nullifiers
    let new_nullifiers = private_call_public_inputs.new_nullifiers;
    let mut siloed_new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_NULLIFIERS_PER_CALL {
        let new_nullifier = new_nullifiers[i];
        if new_nullifier.value != 0 {
            let siloed_note_hash = if new_nullifier.note_hash == 0 {
                0
            } else {
                silo_note_hash(storage_contract_address, new_nullifier.note_hash)
            };
            siloed_new_nullifiers.push(
                SideEffectLinkedToNoteHash {
                value: silo_nullifier(storage_contract_address, new_nullifier.value),
                counter: new_nullifier.counter,
                note_hash: siloed_note_hash
            }
            );
        }
    }
    public_inputs.end.new_nullifiers.extend_from_bounded_vec(siloed_new_nullifiers);

    // note hashes
    let new_note_hashes = private_call_public_inputs.new_note_hashes;
    let mut siloed_new_note_hashes: BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_NOTE_HASHES_PER_CALL {
        let new_note_hash = new_note_hashes[i].value;
        if new_note_hash != 0 {
            siloed_new_note_hashes.push(
                SideEffect { value: silo_note_hash(storage_contract_address, new_note_hash), counter: new_note_hashes[i].counter }
            );
        }
    }
    public_inputs.end.new_note_hashes.extend_from_bounded_vec(siloed_new_note_hashes);

    // Call stacks
    // Private call stack.
    let private_call_stack = array_to_bounded_vec(private_call.private_call_stack);
    validate_call_requests(
        private_call_stack,
        private_call_public_inputs.private_call_stack_hashes,
        private_call
    );
    public_inputs.end.private_call_stack.extend_from_bounded_vec(private_call_stack);
    // Public call stack.
    let public_call_stack = array_to_bounded_vec(private_call.public_call_stack);
    validate_call_requests(
        public_call_stack,
        private_call_public_inputs.public_call_stack_hashes,
        private_call
    );
    public_inputs.end.public_call_stack.extend_from_bounded_vec(public_call_stack);

    // new l2 to l1 messages
    let new_l2_to_l1_msgs = private_call_public_inputs.new_l2_to_l1_msgs;
    let mut new_l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new();
    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_CALL {
        let msg = new_l2_to_l1_msgs[i];
        if !is_empty(msg) {
            let new_l2_to_l1_msgs = compute_l2_to_l1_hash(
                storage_contract_address,
                private_call_public_inputs.version,
                private_call_public_inputs.chain_id,
                msg
            );
            new_l2_to_l1_msgs_to_insert.push(new_l2_to_l1_msgs)
        }
    }
    public_inputs.end.new_l2_to_l1_msgs.extend_from_bounded_vec(new_l2_to_l1_msgs_to_insert);

    // logs hashes
    // See the following thread if not clear:
    // https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426
    let previous_encrypted_logs_hash = public_inputs.end.encrypted_logs_hash;
    let current_encrypted_logs_hash = private_call_public_inputs.encrypted_logs_hash;
    public_inputs.end.encrypted_logs_hash = compute_logs_hash(previous_encrypted_logs_hash,current_encrypted_logs_hash);
    let previous_unencrypted_logs_hash = public_inputs.end.unencrypted_logs_hash;
    let current_unencrypted_logs_hash = private_call_public_inputs.unencrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = compute_logs_hash(previous_unencrypted_logs_hash,current_unencrypted_logs_hash);

    // Add log preimages lengths from current iteration to accumulated lengths
    public_inputs.end.encrypted_log_preimages_length = public_inputs.end.encrypted_log_preimages_length +
                                                           private_call_public_inputs.encrypted_log_preimages_length;
    public_inputs.end.unencrypted_log_preimages_length = public_inputs.end.unencrypted_log_preimages_length + private_call_public_inputs.unencrypted_log_preimages_length;
}

pub fn contract_logic(private_call: PrivateCallData) {
    let contract_address = private_call.call_stack_item.contract_address;

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062): Why is this using a hash function from the stdlib::recursion namespace
    let private_call_vk_hash = stdlib_recursion_verification_key_compress_native_vk(private_call.vk);

    assert(!contract_address.is_zero(), "contract address cannot be zero");
    // std::println(f"contract_address={contract_address}");
    // std::println(f"private_call_vk_hash={private_call_vk_hash}");

    // Recompute the contract class id
    let computed_private_functions_root = private_functions_root_from_siblings(
        private_call.call_stack_item.function_data.selector,
        private_call_vk_hash,
        private_call.function_leaf_membership_witness.leaf_index,
        private_call.function_leaf_membership_witness.sibling_path
    );
    // std::println(f"computed_private_functions_root={computed_private_functions_root}");

    let computed_contract_class_id = ContractClassId::compute(
        private_call.contract_class_artifact_hash,
        computed_private_functions_root,
        private_call.contract_class_public_bytecode_commitment
    );
    // std::println(f"computed_contract_class_id={computed_contract_class_id}");

    // Recompute contract address using the preimage which includes the class_id
    let computed_partial_address = PartialAddress::compute_from_salted_initialization_hash(
        computed_contract_class_id,
        private_call.salted_initialization_hash
    );
    // std::println(f"computed_partial_address={computed_partial_address}");

    let computed_address = AztecAddress::compute(private_call.public_keys_hash, computed_partial_address);
    // std::println(f"computed_address={computed_address}");

    assert(computed_address.eq(contract_address), "computed contract address does not match expected one");
}

pub fn validate_previous_kernel_values(end: PrivateAccumulatedData) {
    assert(end.new_nullifiers[0].value != 0, "The 0th nullifier in the accumulated nullifier array is zero");
}

pub fn validate_call_against_request(private_call: PrivateCallData, request: CallRequest) {
    let call_stack_item = private_call.call_stack_item;
    assert(
        request.hash == call_stack_item.hash(), "calculated private_call_hash does not match provided private_call_hash at the top of the call stack"
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
