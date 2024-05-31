use crate::{
    context::{inputs::PrivateContextInputs, packed_returns::PackedReturns},
    messaging::process_l1_to_l2_message,
    hash::{hash_args_array, ArgsHasher, compute_unencrypted_log_hash},
    keys::constants::{NULLIFIER_INDEX, OUTGOING_INDEX, NUM_KEY_TYPES, sk_generators},
    note::{note_interface::NoteInterface, utils::compute_note_hash_for_insertion},
    oracle::{
    key_validation_request::get_key_validation_request, arguments, returns::pack_returns,
    call_private_function::call_private_function_internal, header::get_header_at,
    logs::{
    emit_encrypted_log, emit_encrypted_note_log, compute_encrypted_log,
    emit_contract_class_unencrypted_log_private_internal, emit_unencrypted_log_private_internal
},
    logs_traits::{LensForEncryptedLog, ToBytesForUnencryptedLog},
    enqueue_public_function_call::{
    enqueue_public_function_call_internal, set_public_teardown_function_call_internal,
    parse_public_call_stack_item_from_oracle
}
}
};
use dep::protocol_types::{
    hash::sha256_to_field,
    abis::{
    caller_context::CallerContext, function_selector::FunctionSelector,
    max_block_number::MaxBlockNumber,
    validation_requests::{KeyValidationRequest, KeyValidationRequestAndGenerator},
    private_call_request::PrivateCallRequest, private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_call_stack_item::PublicCallStackItem, read_request::ReadRequest, note_hash::NoteHash,
    nullifier::Nullifier, log_hash::{LogHash, NoteLogHash, EncryptedLogHash}
},
    address::{AztecAddress, EthAddress},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL, MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_ENCRYPTED_LOGS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL,
    MAX_NOTE_ENCRYPTED_LOGS_PER_CALL
},
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint, header::Header,
    messaging::l2_to_l1_message::L2ToL1Message, utils::reader::Reader, traits::{is_empty, Empty},
    utils::arrays::find_index
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
    key_validation_requests_and_generators: BoundedVec<KeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_CALL>,

    new_note_hashes: BoundedVec<NoteHash, MAX_NEW_NOTE_HASHES_PER_CALL>,
    new_nullifiers: BoundedVec<Nullifier, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_requests : BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack_hashes : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    public_teardown_function_hash: Field,
    new_l2_to_l1_msgs : BoundedVec<L2ToL1Message, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    // docs:end:private-context

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    note_encrypted_logs_hashes: BoundedVec<NoteLogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_CALL>,
    encrypted_logs_hashes: BoundedVec<EncryptedLogHash, MAX_ENCRYPTED_LOGS_PER_CALL>,
    unencrypted_logs_hashes: BoundedVec<LogHash, MAX_UNENCRYPTED_LOGS_PER_CALL>,

    // Contains the last key validation request for each key type. This is used to cache the last request and avoid
    // fetching the same request multiple times.
    // The index of the array corresponds to the key type (0 nullifier, 1 incoming, 2 outgoing, 3 tagging).
    last_key_validation_requests: [Option<KeyValidationRequest>; NUM_KEY_TYPES],
}

impl PrivateContext {
    pub fn new(inputs: PrivateContextInputs, args_hash: Field) -> PrivateContext {
        PrivateContext {
            inputs,
            side_effect_counter: inputs.start_side_effect_counter + 1,
            min_revertible_side_effect_counter: 0,
            is_fee_payer: false,
            args_hash,
            return_hash: 0,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            key_validation_requests_and_generators: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            historical_header: inputs.historical_header,
            private_call_requests: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            new_l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            last_key_validation_requests: [Option::none(); NUM_KEY_TYPES]
        }
    }

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
        pack_returns(returns_hasher.fields);
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
            key_validation_requests_and_generators: self.key_validation_requests_and_generators.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_requests: self.private_call_requests.storage,
            public_call_stack_hashes: self.public_call_stack_hashes.storage,
            public_teardown_function_hash: self.public_teardown_function_hash,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            start_side_effect_counter: self.inputs.start_side_effect_counter,
            end_side_effect_counter: self.side_effect_counter,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            historical_header: self.historical_header,
            tx_context: self.inputs.tx_context
        }
    }

    pub fn set_as_fee_payer(&mut self) {
        dep::protocol_types::debug_log::debug_log_format("Setting {0} as fee payer", [self.this_address().to_field()]);
        self.is_fee_payer = true;
    }

    pub fn end_setup(&mut self) {
        dep::protocol_types::debug_log::debug_log_format(
            "Ending setup at counter {0}",
            [self.side_effect_counter as Field]
        );
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

    pub fn request_nsk_app(&mut self, npk_m_hash: Field) -> Field {
        self.request_sk_app(npk_m_hash, NULLIFIER_INDEX)
    }

    pub fn request_ovsk_app(&mut self, ovpk_m_hash: Field) -> Field {
        self.request_sk_app(ovpk_m_hash, OUTGOING_INDEX)
    }

    fn request_sk_app(&mut self, pk_m_hash: Field, key_index: Field) -> Field {
        let cached_request = self.last_key_validation_requests[key_index].unwrap_or(KeyValidationRequest::empty());

        if cached_request.pk_m.hash() == pk_m_hash {
            // We get a match so the cached request is the latest one 
            cached_request.sk_app
        } else {
            // We didn't get a match meaning the cached result is stale. We fetch new values from oracle and instruct
            // protocol circuits to validate them by storing the validation request in context.
            let request = get_key_validation_request(pk_m_hash, key_index);
            let request_and_generator = KeyValidationRequestAndGenerator { request, sk_app_generator: sk_generators[key_index] };
            // We constrain that the pk_m_hash matches the one in the request (otherwise we could get an arbitrary
            // valid key request and not the one corresponding to pk_m_hash).
            assert(request.pk_m.hash() == pk_m_hash);
            self.key_validation_requests_and_generators.push(request_and_generator);
            self.last_key_validation_requests[key_index] = Option::some(request);
            request.sk_app
        }
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
        let counter = self.next_counter();
        let log_slice = log.to_be_bytes_arr();
        let log_hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
        // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
        let len = 44 + log_slice.len().to_field();
        let side_effect = LogHash { value: log_hash, counter, length: len };
        self.unencrypted_logs_hashes.push(side_effect);
        // call oracle
        let _void = emit_unencrypted_log_private_internal(contract_address, event_selector, log, counter);
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
        // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
        let len = 44 + N * 32;
        let side_effect = LogHash { value: log_hash, counter, length: len };
        self.unencrypted_logs_hashes.push(side_effect);
    }

    // NB: A randomness value of 0 signals that the kernels should not mask the contract address
    // used in siloing later on e.g. 'handshaking' contract w/ known address.
    pub fn encrypt_and_emit_log<N, M>(
        &mut self,
        contract_address: AztecAddress,
        randomness: Field, // Secret random value used later for masked_contract_address
        event_type_id: Field,
        ovpk_m: GrumpkinPoint,
        ivpk_m: GrumpkinPoint,
        preimage: [Field; N]
    ) where [Field; N]: LensForEncryptedLog<N, M> {
        let ovsk_app = self.request_ovsk_app(ovpk_m.hash());

        // We are currently just encrypting it EXACTLY the same way as if it was a note.
        let counter = self.next_counter();
        let encrypted_log: [u8; M] = compute_encrypted_log(
            contract_address,
            randomness,
            event_type_id,
            ovsk_app,
            ovpk_m,
            ivpk_m,
            preimage
        );
        emit_encrypted_log(contract_address, randomness, encrypted_log, counter);
        let len = 32 + 32 + 64 + 48 + 48 + 176 + 64 + (preimage.len() as Field * 32) + 16 + 4;
        let log_hash = sha256_to_field(encrypted_log);
        let side_effect = EncryptedLogHash { value: log_hash, counter, length: len, randomness };
        self.encrypted_logs_hashes.push(side_effect);
    }

    pub fn encrypt_and_emit_note<Note, N, NB, M>(
        &mut self,
        contract_address: AztecAddress,
        storage_slot: Field,
        note_type_id: Field,
        ovpk_m: GrumpkinPoint,
        ivpk_m: GrumpkinPoint,
        note: Note
    ) where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
        let note_hash_counter = note.get_header().note_hash_counter;
        let note_exists_index = find_index(
            self.new_note_hashes.storage,
            |n: NoteHash| n.counter == note_hash_counter
        );
        assert(
            note_exists_index != MAX_NEW_NOTE_HASHES_PER_CALL, "Can only emit a note log for an existing note."
        );
        let preimage = note.serialize_content();
        let counter = self.next_counter();

        let ovsk_app = self.request_ovsk_app(ovpk_m.hash());

        // TODO(#1139 | #6408): perform encryption in the circuit
        let encrypted_log: [u8; M] = compute_encrypted_log(
            contract_address,
            storage_slot,
            note_type_id,
            ovsk_app,
            ovpk_m,
            ivpk_m,
            preimage
        );
        emit_encrypted_note_log(note_hash_counter, encrypted_log, counter);

        // Current unoptimized size of the encrypted log
        // incoming_tag (32 bytes)
        // outgoing_tag (32 bytes)
        // eph_pk (64 bytes)
        // incoming_header (48 bytes)
        // outgoing_header (48 bytes)
        // outgoing_body (176 bytes)
        // incoming_body_fixed (64 bytes)
        // incoming_body_variable (N * 32 bytes + 16 bytes padding) 
        // len of processed log (4 bytes)
        let len = 32 + 32 + 64 + 48 + 48 + 176 + 64 + (preimage.len() as Field * 32) + 16 + 4;

        let log_hash = sha256_to_field(encrypted_log);
        let side_effect = NoteLogHash { value: log_hash, counter, length: len, note_hash_counter };
        self.note_encrypted_logs_hashes.push(side_effect);
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
        let start_side_effect_counter = self.side_effect_counter;
        let item = call_private_function_internal(
            contract_address,
            function_selector,
            args_hash,
            start_side_effect_counter,
            is_static_call,
            is_delegate_call
        );

        assert_eq(item.public_inputs.call_context.side_effect_counter, start_side_effect_counter);
        assert_eq(item.public_inputs.start_side_effect_counter, start_side_effect_counter);
        let end_side_effect_counter = item.public_inputs.end_side_effect_counter;
        self.side_effect_counter = end_side_effect_counter + 1;

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

        // Assert that the call context of the call generated by the oracle matches our request.
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

        let mut caller_context = CallerContext::empty();
        caller_context.is_static_call = self.inputs.call_context.is_static_call;
        if is_delegate_call {
            caller_context.msg_sender = self.inputs.call_context.msg_sender;
            caller_context.storage_contract_address = self.inputs.call_context.storage_contract_address;
        }
        self.private_call_requests.push(
            PrivateCallRequest { hash: item.hash(), caller_context, start_side_effect_counter, end_side_effect_counter }
        );

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
            key_validation_requests_and_generators: BoundedVec::new(),
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            private_call_requests: BoundedVec::new(),
            public_call_stack_hashes: BoundedVec::new(),
            public_teardown_function_hash: 0,
            new_l2_to_l1_msgs: BoundedVec::new(),
            historical_header: Header::empty(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            last_key_validation_requests: [Option::none(); NUM_KEY_TYPES]
        }
    }
}
