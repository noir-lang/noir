use dep::types::{
    abis::{
        call_request::CallRequest,
        combined_accumulated_data::CombinedAccumulatedData,
        complete_address::CompleteAddress,
        function_data::FunctionData,
        kernel_circuit_public_inputs::KernelCircuitPublicInputsBuilder,
        membership_witness::ReadRequestMembershipWitness,
        new_contract_data::NewContractData,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        private_kernel::private_call_data::PrivateCallData,
        previous_kernel_data::PreviousKernelData,
    },
    address::{AztecAddress, EthAddress},
    contrakt::deployment_data::ContractDeploymentData,
    constants::{
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_READ_REQUESTS_PER_CALL,
    },
    hash::{
        compute_constructor_hash,
        compute_l2_to_l1_hash,
        compute_logs_hash,
        compute_new_contract_address_hash,
        contract_tree_root_from_siblings,
        function_tree_root_from_siblings,
        read_request_root_from_siblings,
        silo_commitment,
        silo_nullifier,
        stdlib_recursion_verification_key_compress_native_vk,
    },
    utils::{
        arrays::{
            array_length,
            array_to_bounded_vec,
            is_empty_array,
            struct_array_to_bounded_vec,
            validate_array,
        },
        bounded_vec::BoundedVec,
    },
};

pub fn validate_arrays(app_public_inputs : PrivateCircuitPublicInputs) {
    // Each of the following arrays is expected to be zero-padded.
    // In addition, some of the following arrays (new_commitments, etc...) are passed
    // to push_array_to_array() routines which rely on the passed arrays to be well-formed.
    
    validate_array(app_public_inputs.return_values);
    validate_array(app_public_inputs.read_requests);
    validate_array(app_public_inputs.pending_read_requests);
    validate_array(app_public_inputs.new_commitments);
    validate_array(app_public_inputs.new_nullifiers);
    validate_array(app_public_inputs.nullified_commitments);
    validate_array(app_public_inputs.private_call_stack_hashes);
    validate_array(app_public_inputs.public_call_stack_hashes);
    validate_array(app_public_inputs.new_l2_to_l1_msgs);
    
    // encrypted_logs_hash and unencrypted_logs_hash have their own integrity checks.
}

// Validate all read requests against the historical private data root.
// Use their membership witnesses to do so. If the historical root is not yet
// initialized, initialize it using the first read request here (if present).
//
// More info here:
// - https://discourse.aztec.network/t/to-read-or-not-to-read/178
// - https://discourse.aztec.network/t/spending-notes-which-havent-yet-been-inserted/180
pub fn validate_read_requests(
    historical_note_hash_tree_root: Field,
    read_requests: [Field; MAX_READ_REQUESTS_PER_CALL],
    read_request_membership_witnesses: [ReadRequestMembershipWitness; MAX_READ_REQUESTS_PER_CALL],
) {
    // membership witnesses must resolve to the same private data root
    // for every request in all kernel iterations
    for rr_idx in 0..MAX_READ_REQUESTS_PER_CALL {
        let read_request = read_requests[rr_idx];
        let witness = read_request_membership_witnesses[rr_idx];
        
        // A pending commitment is the one that is not yet added to note hash tree
        // A "transient read" is when we try to "read" a pending commitment within a transaction
        // between function calls, as opposed to reading the outputs of a previous transaction
        // which is a "pending read".
        // A transient read is when we try to "read" a pending commitment
        // We determine if it is a transient read depending on the leaf index from the membership witness
        // Note that the Merkle membership proof would be null and void in case of an transient read
        // but we use the leaf index as a placeholder to detect a 'pending note read'.
    
        if (read_request != 0) & (witness.is_transient == false) {
            let root_for_read_request = read_request_root_from_siblings(read_request, witness.leaf_index, witness.sibling_path);
            assert(root_for_read_request == historical_note_hash_tree_root, "note hash tree root mismatch");
            
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1354): do we need to enforce
            // that a non-transient read_request was derived from the proper/current contract address?
        }
    }
}

pub fn initialize_end_values(previous_kernel : PreviousKernelData, public_inputs : &mut KernelCircuitPublicInputsBuilder) {
    public_inputs.constants = previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    let start = previous_kernel.public_inputs.end;

    public_inputs.end.read_requests = array_to_bounded_vec(start.read_requests);
    public_inputs.end.new_commitments = array_to_bounded_vec(start.new_commitments);
    public_inputs.end.new_nullifiers = array_to_bounded_vec(start.new_nullifiers);
    public_inputs.end.nullified_commitments = array_to_bounded_vec(start.nullified_commitments);

    public_inputs.end.private_call_stack = struct_array_to_bounded_vec(start.private_call_stack, |c: CallRequest| c.is_empty(), CallRequest::empty());
    public_inputs.end.public_call_stack = struct_array_to_bounded_vec(start.public_call_stack, |c: CallRequest| c.is_empty(), CallRequest::empty());
    public_inputs.end.new_l2_to_l1_msgs = array_to_bounded_vec(start.new_l2_to_l1_msgs);

    public_inputs.end.encrypted_logs_hash = start.encrypted_logs_hash;
    public_inputs.end.unencrypted_logs_hash = start.unencrypted_logs_hash;

    public_inputs.end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    public_inputs.end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;

    public_inputs.end.optionally_revealed_data = start.optionally_revealed_data;
    public_inputs.end.new_contracts = struct_array_to_bounded_vec(start.new_contracts, |ncd: NewContractData| ncd.is_empty(), NewContractData::default());
}

fn perform_static_call_checks(private_call: PrivateCallData) {
    let public_inputs = private_call.call_stack_item.public_inputs;
    let is_static_call = public_inputs.call_context.is_static_call;
    if is_static_call {
        // No state changes are allowed for static calls:
        assert(is_empty_array(public_inputs.new_commitments), "new_commitments must be empty for static calls");
        assert(is_empty_array(public_inputs.new_nullifiers), "new_nullifiers must be empty for static calls");
    }
}

fn is_valid_caller(request: CallRequest, private_call: PrivateCallData) -> bool {
    let call_context = private_call.call_stack_item.public_inputs.call_context;
    let valid_caller_context = request.caller_context.msg_sender.eq(call_context.msg_sender)
        & request.caller_context.storage_contract_address.eq(call_context.storage_contract_address);
    request.caller_contract_address.eq(private_call.call_stack_item.contract_address)
        & (request.caller_context.is_empty() | valid_caller_context)
}

fn validate_call_requests<N>(
    call_requests: BoundedVec<CallRequest, N>,
    hashes: [Field; N],
    private_call: PrivateCallData,
) {
    assert_eq(array_length(hashes), call_requests.len(), "call requests length does not match the expected length");
    for i in 0..N {
        let hash = hashes[i];
        if hash != 0 {
            let request = call_requests.get_unchecked(i);
            assert_eq(request.hash, hash, "call stack hash does not match call request hash");
            assert(is_valid_caller(request, private_call), "invalid caller");
        }
    }
}

pub fn update_end_values(private_call: PrivateCallData, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(private_call);

    let private_call_public_inputs = private_call.call_stack_item.public_inputs;

    let read_requests = private_call_public_inputs.read_requests;
    let read_request_membership_witnesses = private_call.read_request_membership_witnesses;

    // don't update pending_read_requests, because those just get passed through without any change

    let new_commitments = private_call_public_inputs.new_commitments;
    let new_nullifiers = private_call_public_inputs.new_nullifiers;
    let nullified_commitments = private_call_public_inputs.nullified_commitments;

    let storage_contract_address = private_call_public_inputs.call_context.storage_contract_address;
    
    // Transient read requests and witnesses are accumulated in public_inputs.end
    // We silo the read requests (domain separation per contract address)
    let mut siloed_read_requests: BoundedVec<Field, MAX_READ_REQUESTS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_READ_REQUESTS_PER_CALL {
        let read_request = read_requests[i];
        let witness = read_request_membership_witnesses[i];
        if witness.is_transient & (read_request != 0) {  // only forward transient to public inputs
            siloed_read_requests.push(silo_commitment(storage_contract_address, read_request))
        }
    }
    public_inputs.end.read_requests.push_vec(siloed_read_requests);

    // Enhance commitments and nullifiers with domain separation whereby domain is the contract.
    //
    // nullifiers
    let mut siloed_new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_NULLIFIERS_PER_CALL {
        let new_nullifier = new_nullifiers[i];
        if new_nullifier != 0 {
            siloed_new_nullifiers.push(silo_nullifier(storage_contract_address, new_nullifier));
        }
    }
    public_inputs.end.new_nullifiers.push_vec(siloed_new_nullifiers);

    // commitments
    let mut siloed_new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_COMMITMENTS_PER_CALL {
        let new_commitment = new_commitments[i];
        if new_commitment != 0 {
            siloed_new_commitments.push(silo_commitment(storage_contract_address, new_commitment));
        }
    }
    public_inputs.end.new_commitments.push_vec(siloed_new_commitments);

    // nullified commitments (for matching transient nullifiers to transient commitments)
    // Since every new_nullifiers entry is paired with a nullified_commitment, EMPTY
    // is used here for nullified_commitments of persistable nullifiers. EMPTY will still
    // take up a slot in the nullified_commitments array so that the array lines up properly
    // with new_nullifiers. This is necessary since the constant-size circuit-array functions
    // we use assume that the first 0-valued array entry designates the end of the array.
    let mut siloed_nullified_commitments: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_NULLIFIERS_PER_CALL {
        // TODO(David): There are two ways to specify that a nullified commitment is empty.
        // Either with 0 or the special EMPTY_NULLIFIED_COMMITMENT constant
        // We do not silo when the commitment is either one of these empty states.

        let nullified_commitment_is_zero = nullified_commitments[i] == 0;
        let nullified_commitment_is_empty_constant = nullified_commitments[i] == EMPTY_NULLIFIED_COMMITMENT;
        let siloed_nullified_commitment = if nullified_commitment_is_zero {
            0
        } else if nullified_commitment_is_empty_constant {
            EMPTY_NULLIFIED_COMMITMENT
        } else {
            silo_commitment(storage_contract_address, nullified_commitments[i])
        };

        if siloed_nullified_commitment != 0 {
            siloed_nullified_commitments.push(siloed_nullified_commitment);
        }
    }
    // TODO(Kev): These operations should be carefully checked with the cpp array_push code
    public_inputs.end.nullified_commitments.push_vec(siloed_nullified_commitments);

    // Call stacks
    // Private call stack.
    let private_call_stack = struct_array_to_bounded_vec(private_call.private_call_stack, |c: CallRequest| c.is_empty(), CallRequest::empty());
    validate_call_requests(private_call_stack, private_call_public_inputs.private_call_stack_hashes, private_call);
    public_inputs.end.private_call_stack.push_vec(private_call_stack);
    // Public call stack.
    let public_call_stack = struct_array_to_bounded_vec(private_call.public_call_stack, |c: CallRequest| c.is_empty(), CallRequest::empty());
    validate_call_requests(public_call_stack, private_call_public_inputs.public_call_stack_hashes, private_call);
    public_inputs.end.public_call_stack.push_vec(public_call_stack);

    // new l2 to l1 messages
    let portal_contract_address = private_call.portal_contract_address;
    let new_l2_to_l1_msgs = private_call_public_inputs.new_l2_to_l1_msgs;
    let mut new_l2_to_l1_msgs_to_insert : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> = BoundedVec::new(0);
    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_CALL {
        let msg_is_zero = new_l2_to_l1_msgs[i] == 0;
        if msg_is_zero == false {
            let new_l2_to_l1_msgs = compute_l2_to_l1_hash(storage_contract_address,
                                                                           private_call_public_inputs.version,
                                                                           portal_contract_address,
                                                                           private_call_public_inputs.chain_id,
                                                                           new_l2_to_l1_msgs[i]);
            new_l2_to_l1_msgs_to_insert.push(new_l2_to_l1_msgs)
        }
    }
    public_inputs.end.new_l2_to_l1_msgs.push_vec(new_l2_to_l1_msgs_to_insert);

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

pub fn contract_logic(private_call : PrivateCallData, public_inputs : &mut KernelCircuitPublicInputsBuilder, contract_dep_data : ContractDeploymentData, function_data : FunctionData) {
    let private_call_public_inputs = private_call.call_stack_item.public_inputs;
    let portal_contract_address = private_call.portal_contract_address;
    let contract_address = private_call.call_stack_item.contract_address;

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062): Why is this using a hash function from the stdlib::recursion namespace
    let private_call_vk_hash =
        stdlib_recursion_verification_key_compress_native_vk(private_call.vk);

    let is_contract_deployment = public_inputs.constants.tx_context.is_contract_deployment_tx;

    // input storage contract address must be 0 if its a constructor call and non-zero otherwise
    if is_contract_deployment {
        //TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062) use locally computed private_call_vk_hash here
        let constructor_hash = compute_constructor_hash(function_data, private_call_public_inputs.args_hash, contract_dep_data.constructor_vk_hash);
        let new_contract_address = CompleteAddress::compute(
            contract_dep_data.deployer_public_key,
            contract_dep_data.contract_address_salt,
            contract_dep_data.function_tree_root,
            constructor_hash,
        ).address;
        
        let new_contract_data = NewContractData {
            contract_address: new_contract_address,
            portal_contract_address,
            function_tree_root: contract_dep_data.function_tree_root,
        };
        public_inputs.end.new_contracts.push(new_contract_data);
        
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062) VKs are mocked out for now
        // assert(contract_dep_data.constructor_vk_hash == private_call_vk_hash, "constructor_vk_hash doesn't match private_call_vk_hash");
        
        assert(contract_address.eq(new_contract_address), "contract address supplied does not match derived address");

        let new_contract_address_nullifier = compute_new_contract_address_hash(new_contract_address);

        public_inputs.end.new_nullifiers.push(new_contract_address_nullifier);
    } else {
        // non-contract deployments must specify contract address being interacted with
        assert(!contract_address.is_zero(), "contract address cannot be zero");

         /* We need to compute the root of the contract tree, starting from the function's VK:
         * - Compute the vk_hash (done above)
         * - Compute the function_leaf: hash(function_selector, is_internal, is_private, vk_hash, acir_hash)
         * - Hash the function_leaf with the function_leaf's sibling_path to get the function_tree_root
         * - Compute the contract_leaf: hash(contract_address, portal_contract_address, function_tree_root)
         * - Hash the contract_leaf with the contract_leaf's sibling_path to get the contract_tree_root
         */

        // The logic below ensures that the contract exists in the contracts tree
        let computed_function_tree_root = function_tree_root_from_siblings(private_call.call_stack_item.function_data.selector,
                                                 private_call.call_stack_item.function_data.is_internal,
                                                 true,  // is_private
                                                 private_call_vk_hash,
                                                 private_call.acir_hash,
                                                 private_call.function_leaf_membership_witness.leaf_index,
                                                 private_call.function_leaf_membership_witness.sibling_path);
        
        let computed_contract_tree_root = contract_tree_root_from_siblings(computed_function_tree_root,
                                                 contract_address,
                                                 portal_contract_address,
                                                 private_call.contract_leaf_membership_witness.leaf_index,
                                                 private_call.contract_leaf_membership_witness.sibling_path);

        let purported_contract_tree_root = private_call.call_stack_item.public_inputs.block_header.contract_tree_root();
        assert_eq(computed_contract_tree_root, purported_contract_tree_root, "computed_contract_tree_root does not match purported_contract_tree_root");
    }
}

pub fn validate_previous_kernel_values(end: CombinedAccumulatedData) {
    assert(end.new_nullifiers[0] != 0, "The 0th nullifier in the accumulated nullifier array is zero");
}

pub fn validate_call_against_request(private_call: PrivateCallData, request: CallRequest) {
    let call_stack_item = private_call.call_stack_item;
    assert(request.hash == call_stack_item.hash(), "calculated private_call_hash does not match provided private_call_hash at the top of the call stack");

    let call_context = call_stack_item.public_inputs.call_context;

    // Ensures that if the function is internal, only the contract itself can call it.
    if call_stack_item.function_data.is_internal {
        assert(call_context.msg_sender.eq(call_context.storage_contract_address), "call is internal but msg_sender is not self");
    }

    if call_context.is_delegate_call {
        let caller_context = request.caller_context;
        assert(!caller_context.is_empty(), "caller context cannot be empty for delegate calls");
        assert(call_context.msg_sender.eq(caller_context.msg_sender), "call stack msg_sender does not match expected msg_sender for delegate calls");
        assert(call_context.storage_contract_address.eq(caller_context.storage_contract_address), "call stack storage address does not match expected contract address for delegate calls");
        assert(!call_stack_item.contract_address.eq(call_context.storage_contract_address),
            "curent contract address must not match storage contract address for delegate calls");
    } else {
        let caller_contract_address = request.caller_contract_address;
        assert(call_context.msg_sender.eq(caller_contract_address), "call stack msg_sender does not match caller contract address");
        assert(call_context.storage_contract_address.eq(call_stack_item.contract_address), "call stack storage address does not match expected contract address");
    }
}
