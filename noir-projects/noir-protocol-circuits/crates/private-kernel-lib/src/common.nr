use dep::std;
use dep::types::{
    abis::{
    call_request::CallRequest, accumulated_data::PrivateAccumulatedData,
    membership_witness::NoteHashReadRequestMembershipWitness,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    private_kernel::private_call_data::PrivateCallData, side_effect::SideEffect
},
    address::{AztecAddress, PartialAddress}, contract_class_id::ContractClassId,
    constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    hash::{private_functions_root_from_siblings, stdlib_recursion_verification_key_compress_native_vk},
    merkle_tree::check_membership, utils::{arrays::{array_length, validate_array}},
    traits::{is_empty, is_empty_array}
};

fn validate_arrays(app_public_inputs: PrivateCircuitPublicInputs) {
    // Each of the following arrays is expected to be zero-padded.
    // In addition, some of the following arrays (new_note_hashes, etc...) are passed
    // to extend_from_array_to_array() routines which rely on the passed arrays to be well-formed.

    validate_array(app_public_inputs.note_hash_read_requests);
    validate_array(app_public_inputs.nullifier_read_requests);
    validate_array(app_public_inputs.nullifier_key_validation_requests);
    validate_array(app_public_inputs.new_note_hashes);
    validate_array(app_public_inputs.new_nullifiers);
    validate_array(app_public_inputs.private_call_stack_hashes);
    validate_array(app_public_inputs.public_call_stack_hashes);
    validate_array(app_public_inputs.new_l2_to_l1_msgs);
    validate_array(app_public_inputs.encrypted_logs_hashes);
    validate_array(app_public_inputs.unencrypted_logs_hashes);
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

fn validate_call_requests<N>(call_requests: [CallRequest; N], hashes: [Field; N], private_call: PrivateCallData) {
    for i in 0..N {
        let hash = hashes[i];
        let request = call_requests[i];
        if hash != 0 {
            assert_eq(request.hash, hash, "call stack hash does not match call request hash");
            assert(is_valid_caller(request, private_call), "invalid caller");
        } else {
            assert(is_empty(request), "call requests length does not match the expected length");
        }
    }
}

// TODO: Move to a seperate file.
pub fn validate_private_call_data(private_call: PrivateCallData) {
    let private_call_public_inputs = private_call.call_stack_item.public_inputs;

    validate_arrays(private_call_public_inputs);

    contract_logic(private_call);

    perform_static_call_checks(private_call);

    // Private call stack.
    validate_call_requests(
        private_call.private_call_stack,
        private_call_public_inputs.private_call_stack_hashes,
        private_call
    );

    // Public call stack.
    validate_call_requests(
        private_call.public_call_stack,
        private_call_public_inputs.public_call_stack_hashes,
        private_call
    );
}

fn contract_logic(private_call: PrivateCallData) {
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
