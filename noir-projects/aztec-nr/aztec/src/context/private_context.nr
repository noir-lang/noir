use crate::{
    context::{inputs::PrivateContextInputs, interface::ContextInterface},
    messaging::process_l1_to_l2_message,
    hash::{hash_args_array, ArgsHasher, compute_encrypted_log_hash, compute_unencrypted_log_hash},
    oracle::{
    arguments, returns, call_private_function::call_private_function_internal, header::get_header_at,
    logs::emit_encrypted_log, logs_traits::{LensForEncryptedLog, ToBytesForUnencryptedLog},
    nullifier_key::{get_nullifier_keys, get_nullifier_keys_with_npk_m_hash, NullifierKeys},
    enqueue_public_function_call::{
    enqueue_public_function_call_internal, set_public_teardown_function_call_internal,
    parse_public_call_stack_item_from_oracle
}
}
};
use dep::protocol_types::{
    abis::{
    function_selector::FunctionSelector, max_block_number::MaxBlockNumber,
    nullifier_key_validation_request::NullifierKeyValidationRequest,
    private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_call_stack_item::PublicCallStackItem, read_request::ReadRequest, note_hash::NoteHash,
    nullifier::Nullifier, side_effect::SideEffect
},
    address::{AztecAddress, EthAddress},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_ENCRYPTED_LOGS_PER_CALL,
    MAX_UNENCRYPTED_LOGS_PER_CALL
},
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, utils::reader::Reader,
    traits::{is_empty, Deserialize, Empty}, hash::poseidon2_hash
};

// When finished, one can call .finish() to convert back to the abi
struct PrivateContext {
    // docs:start:private-context
    inputs: PrivateContextInputs,
    side_effect_counter: u32,

    min_revertible_side_effect_counter: u32,
    is_fee_payer: bool,

    args_hash: Field,
    return_hash: Field,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: BoundedVec<ReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL>,
    nullifier_read_requests: BoundedVec<ReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_CALL>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_note_hashes: BoundedVec<NoteHash, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<Nullifier, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack_hashes : BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    public_teardown_function_hash: Field,
    new_l2_to_l1_msgs : BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    // docs:end:private-context

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    encrypted_logs_hashes: BoundedVec<SideEffect, MAX_ENCRYPTED_LOGS_PER_CALL>,
    unencrypted_logs_hashes: BoundedVec<SideEffect, MAX_UNENCRYPTED_LOGS_PER_CALL>,
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    nullifier_key: Option<NullifierKeys>,
}

impl ContextInterface for PrivateContext {
    fn msg_sender(self) -> AztecAddress {
        self.inputs.call_context.msg_sender
    }

    fn this_address(self) -> AztecAddress {
        self.inputs.call_context.storage_contract_address
    }

    fn chain_id(self) -> Field {
        self.inputs.tx_context.chain_id
    }

    fn version(self) -> Field {
        self.inputs.tx_context.version
    }

    fn selector(self) -> FunctionSelector {
        self.inputs.call_context.function_selector
    }

    fn get_args_hash(self) -> Field {
        self.args_hash
    }

    fn push_new_note_hash(&mut self, note_hash: Field) {
        self.new_note_hashes.push(NoteHash { value: note_hash, counter: self.next_counter() });
    }

    fn push_new_nullifier(&mut self, nullifier: Field, nullified_note_hash: Field) {
        self.new_nullifiers.push(Nullifier { value: nullifier, note_hash: nullified_note_hash, counter: self.next_counter() });
    }
}

impl PrivateContext {
    pub fn new(inputs: PrivateContextInputs, args_hash: Field) -> PrivateContext {
        let side_effect_counter = inputs.start_side_effect_counter;
        let mut min_revertible_side_effect_counter = 0;
        if is_empty(inputs.call_context.msg_sender) {
            min_revertible_side_effect_counter = side_effect_counter;
        }
        PrivateContext {
            inputs,
            side_effect_counter,
            min_revertible_side_effect_counter,
            is_fee_payer: false,
            args_hash,
            return_hash: 0,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_key_validation_requests: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            historical_header: inputs.historical_header,
            private_call_stack_hashes: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            new_l2_to_l1_msgs: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            nullifier_key: Option::none()
        }
    }

    // Returns the header of a block whose state is used during private execution (not the block the transaction is
    // included in).
    fn get_header(self) -> Header {
        self.historical_header
    }

    // Returns the header of an arbitrary block whose block number is less than or equal to the block number
    // of historical header.
    pub fn get_header_at(self, block_number: u32) -> Header {
        get_header_at(block_number, self)
    }

    pub fn set_return_hash(&mut self, returns_hasher: ArgsHasher) {
        returns::pack_returns(returns_hasher.fields);
        self.return_hash = returns_hasher.hash();
    }

    pub fn finish(self) -> PrivateCircuitPublicInputs {
        PrivateCircuitPublicInputs {
            call_context: self.inputs.call_context,
            args_hash: self.args_hash,
            returns_hash: self.return_hash,
            min_revertible_side_effect_counter: self.min_revertible_side_effect_counter,
            is_fee_payer: self.is_fee_payer,
            max_block_number: self.max_block_number,
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack_hashes: self.private_call_stack_hashes.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            public_teardown_function_hash: self.public_teardown_function_hash,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.inputs.start_side_effect_counter,
            end_side_effect_counter: self.side_effect_counter,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length + 4,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length + 4,
            historical_header: self.historical_header,
            tx_context: self.inputs.tx_context
        }
    }

    pub fn set_as_fee_payer(&mut self) {
        self.is_fee_payer = true;
    }

    pub fn end_setup(&mut self) {
        self.min_revertible_side_effect_counter = self.side_effect_counter;
    }

    // docs:start:max-block-number
    pub fn set_tx_max_block_number(&mut self, max_block_number: u32) {
        // docs:end:max-block-number
        self.max_block_number = MaxBlockNumber::min_with_u32(self.max_block_number, max_block_number);
    }

    pub fn push_note_hash_read_request(&mut self, note_hash: Field) {
        let side_effect = ReadRequest { value: note_hash, counter: self.next_counter() };
        self.note_hash_read_requests.push(side_effect);
    }

    pub fn push_nullifier_read_request(&mut self, nullifier: Field) {
        let request = ReadRequest { value: nullifier, counter: self.next_counter() };
        self.nullifier_read_requests.push(request);
    }

    pub fn request_app_nullifier_secret_key(&mut self, account: AztecAddress) -> Field {
        let keys = if self.nullifier_key.is_none() {
            let keys = get_nullifier_keys(account);
            let request = NullifierKeyValidationRequest {
                master_nullifier_public_key: keys.master_nullifier_public_key,
                app_nullifier_secret_key: keys.app_nullifier_secret_key
            };
            self.nullifier_key_validation_requests.push(request);
            self.nullifier_key = Option::some(keys);
            keys
        } else {
            let keys = self.nullifier_key.unwrap_unchecked();
            // If MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL is larger than 1, need to update the way the key pair is cached.
            assert(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL == 1);
            assert(keys.account == account, "Cannot query nullifier key for more than one account per call");
            keys
        };
        keys.app_nullifier_secret_key
    }

    // TODO(#5630) Replace request_app_nullifier_secret_key above with this once we no longer get app nullifier secret key with address
    pub fn request_nsk_app_with_npk_m_hash(&mut self, npk_m_hash: Field) -> Field {
        let keys = if self.nullifier_key.is_none() {
            let keys = get_nullifier_keys_with_npk_m_hash(npk_m_hash);
            let request = NullifierKeyValidationRequest {
                master_nullifier_public_key: keys.master_nullifier_public_key,
                app_nullifier_secret_key: keys.app_nullifier_secret_key
            };
            self.nullifier_key_validation_requests.push(request);
            self.nullifier_key = Option::some(keys);
            keys
        } else {
            let keys = self.nullifier_key.unwrap_unchecked();
            // If MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL is larger than 1, need to update the way the key pair is cached.
            assert(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL == 1);
            keys
        };

        // We have to check if the key that was requested or cached corresponds to the one we request for
        assert_eq(poseidon2_hash(keys.master_nullifier_public_key.serialize()), npk_m_hash);
        keys.app_nullifier_secret_key
    }

    // docs:start:context_message_portal
    pub fn message_portal(&mut self, recipient: EthAddress, content: Field) {
        // docs:end:context_message_portal
        let message = L2ToL1Message { recipient, content, counter: self.next_counter() };
        self.new_l2_to_l1_msgs.push(message);
    }

    // docs:start:context_consume_l1_to_l2_message
    // docs:start:consume_l1_to_l2_message
    pub fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress) {
        // docs:end:context_consume_l1_to_l2_message
        let nullifier = process_l1_to_l2_message(
            self.historical_header.state.l1_to_l2_message_tree.root,
            self.this_address(),
            sender,
            self.chain_id(),
            self.version(),
            content,
            secret
        );

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, 0)
    }
    // docs:end:consume_l1_to_l2_message

    // TODO: We might want to remove this since emitting unencrypted logs from private functions is violating privacy.
    // --> might be a better approach to force devs to make a public function call that emits the log if needed then
    // it would be less easy to accidentally leak information.
    // If we decide to keep this function around would make sense to wait for traits and then merge it with emit_unencrypted_log.
    pub fn emit_unencrypted_log<T, N, M>(&mut self, log: T) where T: ToBytesForUnencryptedLog<N, M> {
        let event_selector = 5; // TODO: compute actual event selector.
        let contract_address = self.this_address();
        let log_slice = log.to_be_bytes_arr();
        let log_hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
        let side_effect = SideEffect { value: log_hash, counter: self.next_counter() };
        self.unencrypted_logs_hashes.push(side_effect);
        // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
        self.unencrypted_log_preimages_length += 44 + log_slice.len().to_field();
        // call oracle
        let _void = emit_unencrypted_log_private_internal(contract_address, event_selector, log, side_effect.counter);
    }

    // This fn exists separately from emit_unencrypted_log because sha hashing the preimage
    // is too large to compile (16,200 fields, 518,400 bytes) => the oracle hashes it
    // It is ONLY used with contract_class_registerer_contract since we already assert correctness:
    // - Contract class -> we will commit to the packed bytecode (currently a TODO)
    // - Private function -> we provide a membership proof
    // - Unconstrained function -> we provide a membership proof
    // Ordinary logs are not protected by the above so this fn shouldn't be called by anything else
    pub fn emit_contract_class_unencrypted_log<N>(&mut self, log: [Field; N]) {
        let event_selector = 5; // TODO: compute actual event selector.
        let contract_address = self.this_address();
        let counter = self.next_counter();
        let log_hash = emit_contract_class_unencrypted_log_private_internal(contract_address, event_selector, log, counter);
        let side_effect = SideEffect { value: log_hash, counter };
        self.unencrypted_logs_hashes.push(side_effect);
        // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
        self.unencrypted_log_preimages_length += 44 + N*32;
    }

    pub fn emit_encrypted_log<N, M, L>(
        &mut self,
        contract_address: AztecAddress,
        storage_slot: Field,
        note_type_id: Field,
        ivpk_m: GrumpkinPoint,
        preimage: [Field; N]
    ) where [Field; N]: LensForEncryptedLog<N, M, L> {
        // TODO(1139): perform encryption in the circuit
        // The oracle call should come last, but we require the encrypted value for now
        let counter = self.next_counter();
        let encrypted_log: [Field; M] = emit_encrypted_log(
            contract_address,
            storage_slot,
            note_type_id,
            ivpk_m,
            preimage,
            counter
        );
        let log_hash = compute_encrypted_log_hash(encrypted_log);
        let side_effect = SideEffect { value: log_hash, counter };
        self.encrypted_logs_hashes.push(side_effect);
        let encrypted_log_byte_len = 112 + 32 * (N + 3);
        // + processed log len (4)
        self.encrypted_log_preimages_length += encrypted_log_byte_len + 4;
    }

    pub fn call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn static_call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, true, false)
    }

    pub fn delegate_call_private_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) -> PackedReturns {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_private_function_with_packed_args(contract_address, function_selector, args_hash, false, true)
    }

    pub fn call_private_function_no_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, false, false)
    }

    pub fn static_call_private_function_no_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, true, false)
    }

    pub fn delegate_call_private_function_no_args<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) -> PackedReturns {
        self.call_private_function_with_packed_args(contract_address, function_selector, 0, false, true)
    }

    pub fn call_private_function_with_packed_args<RETURNS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) -> PackedReturns {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let item = call_private_function_internal(
            contract_address,
            function_selector,
            args_hash,
            self.side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        assert_eq(item.public_inputs.call_context.side_effect_counter, self.side_effect_counter);
        assert_eq(item.public_inputs.start_side_effect_counter, self.side_effect_counter);
        self.side_effect_counter = item.public_inputs.end_side_effect_counter + 1;

        // TODO (fees) figure out why this crashes the prover and enable it
        // we need this in order to pay fees inside child call contexts
        // assert(
        //     (item.public_inputs.min_revertible_side_effect_counter == 0 as u32)
        //     | (item.public_inputs.min_revertible_side_effect_counter
        //         > self.min_revertible_side_effect_counter)
        // );

        // if item.public_inputs.min_revertible_side_effect_counter
        //     > self.min_revertible_side_effect_counter {
        //     self.min_revertible_side_effect_counter = item.public_inputs.min_revertible_side_effect_counter;
        // }

        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        assert(item.public_inputs.call_context.is_delegate_call == is_delegate_call);
        assert(item.public_inputs.call_context.is_static_call == is_static_call);

        if (is_delegate_call) {
            // For delegate calls, we also constrain the execution context address for the nested call to be equal to our address.
            assert(
                item.public_inputs.call_context.storage_contract_address.eq(self.inputs.call_context.storage_contract_address)
            );
            assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.msg_sender));
        } else {
            // For non-delegate calls, we also constrain the execution context address for the nested call to be equal to the address we called.
            assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));
            assert(
                item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address)
            );
        }

        self.private_call_stack_hashes.push(item.hash());

        PackedReturns::new(item.public_inputs.returns_hash)
    }

    pub fn call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn static_call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, true, false)
    }

    pub fn delegate_call_public_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.call_public_function_with_packed_args(contract_address, function_selector, args_hash, false, true)
    }

    pub fn call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, false)
    }

    pub fn static_call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, true, false)
    }

    pub fn delegate_call_public_function_no_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector
    ) {
        self.call_public_function_with_packed_args(contract_address, function_selector, 0, false, true)
    }

    pub fn call_public_function_with_packed_args(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let fields = enqueue_public_function_call_internal(
            contract_address,
            function_selector,
            args_hash,
            self.side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        let item = parse_public_call_stack_item_from_oracle(fields);
        self.validate_call_stack_item_from_oracle(
            item,
            contract_address,
            function_selector,
            args_hash,
            is_static_call,
            is_delegate_call
        );

        self.side_effect_counter = self.side_effect_counter + 1;
        self.public_call_stack_hashes.push(item.hash());
    }

    pub fn set_public_teardown_function<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field; ARGS_COUNT]
    ) {
        let args_hash = hash_args_array(args);
        assert(args_hash == arguments::pack_arguments_array(args));
        self.set_public_teardown_function_with_packed_args(contract_address, function_selector, args_hash, false, false)
    }

    pub fn set_public_teardown_function_with_packed_args<ARGS_COUNT>(
        &mut self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        let mut is_static_call = is_static_call | self.inputs.call_context.is_static_call;
        let fields = set_public_teardown_function_call_internal(
            contract_address,
            function_selector,
            args_hash,
            self.side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        let item = parse_public_call_stack_item_from_oracle(fields);
        self.validate_call_stack_item_from_oracle(
            item,
            contract_address,
            function_selector,
            args_hash,
            is_static_call,
            is_delegate_call
        );

        self.side_effect_counter = self.side_effect_counter + 1;
        self.public_teardown_function_hash = item.hash();
    }

    fn validate_call_stack_item_from_oracle(
        self,
        item: PublicCallStackItem,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args_hash: Field,
        is_static_call: bool,
        is_delegate_call: bool
    ) {
        assert(contract_address.eq(item.contract_address));
        assert(function_selector.eq(item.function_data.selector));

        assert_eq(item.public_inputs.call_context.side_effect_counter, self.side_effect_counter);

        assert(args_hash == item.public_inputs.args_hash);

        // Assert that the call context of the enqueued call generated by the oracle matches our request.
        assert(item.public_inputs.call_context.is_delegate_call == is_delegate_call);
        assert(item.public_inputs.call_context.is_static_call == is_static_call);

        if (is_delegate_call) {
            // For delegate calls, we also constrain the execution context address for the nested call to be equal to our address.
            assert(
                item.public_inputs.call_context.storage_contract_address.eq(self.inputs.call_context.storage_contract_address)
            );
            assert(item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.msg_sender));
        } else {
            // For non-delegate calls, we also constrain the execution context address for the nested call to be equal to the address we called.
            assert(item.public_inputs.call_context.storage_contract_address.eq(contract_address));
            assert(
                item.public_inputs.call_context.msg_sender.eq(self.inputs.call_context.storage_contract_address)
            );
        }
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.side_effect_counter;
        self.side_effect_counter += 1;
        counter
    }
}

impl Empty for PrivateContext {
    fn empty() -> Self {
        PrivateContext {
            inputs: PrivateContextInputs::empty(),
            side_effect_counter: 0 as u32,
            min_revertible_side_effect_counter: 0 as u32,
            is_fee_payer: false,
            args_hash: 0,
            return_hash: 0,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_key_validation_requests: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            private_call_stack_hashes : BoundedVec::new(),
            public_call_stack_hashes : BoundedVec::new(),
            public_teardown_function_hash: 0,
            new_l2_to_l1_msgs : BoundedVec::new(),
            historical_header: Header::empty(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            nullifier_key: Option::none(),
        }
    }
}

struct PackedReturns {
  packed_returns: Field,
}

impl PackedReturns {
    pub fn new(packed_returns: Field) -> Self {
        PackedReturns { packed_returns }
    }

    pub fn assert_empty(self) {
        assert_eq(self.packed_returns, 0);
    }

    pub fn raw(self) -> Field {
        self.packed_returns
    }

    pub fn unpack<N>(self) -> [Field; N] {
        let unpacked: [Field; N] = returns::unpack_returns(self.packed_returns);
        assert_eq(self.packed_returns, hash_args_array(unpacked));
        unpacked
    }

    pub fn unpack_into<T, N>(self) -> T where T: Deserialize<N> {
        let unpacked: [Field; N] = self.unpack();
        Deserialize::deserialize(unpacked)
    }
}

#[oracle(emitUnencryptedLog)]
fn emit_unencrypted_log_oracle_private<T>(
    _contract_address: AztecAddress,
    _event_selector: Field,
    _message: T,
    _counter: u32
) -> Field {}

unconstrained pub fn emit_unencrypted_log_private_internal<T>(
    contract_address: AztecAddress,
    event_selector: Field,
    message: T,
    counter: u32
) -> Field {
    emit_unencrypted_log_oracle_private(contract_address, event_selector, message, counter)
}

#[oracle(emitContractClassUnencryptedLog)]
fn emit_contract_class_unencrypted_log_private<N>(
    contract_address: AztecAddress,
    event_selector: Field,
    message: [Field; N],
    counter: u32
) -> Field {}

unconstrained pub fn emit_contract_class_unencrypted_log_private_internal<N>(
    contract_address: AztecAddress,
    event_selector: Field,
    message: [Field; N],
    counter: u32
) -> Field {
    emit_contract_class_unencrypted_log_private(contract_address, event_selector, message, counter)
}

