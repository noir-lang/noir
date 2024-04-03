// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract Test {
    use dep::aztec::prelude::{
        AztecAddress, EthAddress, FunctionSelector, NoteHeader, NoteGetterOptions, NoteViewerOptions,
        PrivateContext, PrivateImmutable, PrivateSet
    };

    use dep::aztec::protocol_types::{
        abis::private_circuit_public_inputs::PrivateCircuitPublicInputs,
        constants::{MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NOTES_PER_PAGE}
    };
    // docs:start:unencrypted_import
    use dep::aztec::prelude::emit_unencrypted_log;
    // docs:end:unencrypted_import

    use dep::aztec::{
        context::{Context, inputs::private_context_inputs::PrivateContextInputs},
        hash::{pedersen_hash, compute_secret_hash, ArgsHasher},
        note::{
        lifecycle::{create_note, destroy_note}, note_getter::{get_notes, view_notes},
        note_getter_options::NoteStatus
    },
        deploy::deploy_contract as aztec_deploy_contract,
        oracle::{
        get_public_key::get_public_key as get_public_key_oracle, context::get_portal_address,
        unsafe_rand::unsafe_rand
    },
        log::emit_unencrypted_log_from_private
    };
    use dep::token_portal_content_hash_lib::{get_mint_private_content_hash, get_mint_public_content_hash};
    use dep::field_note::field_note::FieldNote;
    use dep::value_note::value_note::ValueNote;

    #[event]
    struct ExampleEvent {
        value: Field,
    }

    struct Storage {
        example_constant: PrivateImmutable<FieldNote>,
        example_set: PrivateSet<FieldNote>,
    }

    #[aztec(private)]
    fn get_public_key(address: AztecAddress) -> [Field; 2] {
        let pub_key = get_public_key_oracle(address);

        [pub_key.x, pub_key.y]
    }

    // Get the portal contract address through an oracle call
    #[aztec(private)]
    fn get_portal_contract_address(aztec_address: AztecAddress) -> EthAddress {
        get_portal_address(aztec_address)
    }

    // Get the address of the l1 portal for this contract (taken from the input context)
    #[aztec(private)]
    fn get_this_portal_address() -> EthAddress {
        context.this_portal_address()
    }

    // Get the address of this contract (taken from the input context)
    #[aztec(private)]
    fn get_this_address() -> AztecAddress {
        context.this_address()
    }

    #[aztec(private)]
    fn request_max_block_number(max_block_number: u32, enqueue_public_call: bool) {
        // docs:start:request-max-block-number
        context.request_max_block_number(max_block_number);
        // docs:end:request-max-block-number

        if enqueue_public_call {
            let _ = context.call_public_function(
                context.this_address(),
                FunctionSelector::from_signature("dummy_public_call()"),
                []
            );
        }
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn dummy_public_call() {}

    #[aztec(private)]
    fn call_create_note(value: Field, owner: AztecAddress, storage_slot: Field) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut note = ValueNote::new(value, owner);
        create_note(&mut context, storage_slot, &mut note, true);
    }

    #[aztec(private)]
    fn call_get_notes(storage_slot: Field, active_or_nullified: bool) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteGetterOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let opt_notes: [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

        // We can't get the return value of a private function from the outside world in an end to end test, so we
        // expose it via an unecrypted log instead.
        let value = opt_notes[0].unwrap().value;
        emit_unencrypted_log_from_private(&mut context, value);
    }

    #[aztec(private)]
    fn call_get_notes_many(storage_slot: Field, active_or_nullified: bool) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteGetterOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let opt_notes: [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

        // We can't get the return value of a private function from the outside world in an end to end test, so we
        // expose it via an unecrypted log instead.
        emit_unencrypted_log_from_private(&mut context, opt_notes[0].unwrap().value);
        emit_unencrypted_log_from_private(&mut context, opt_notes[1].unwrap().value);
    }

    unconstrained fn call_view_notes(storage_slot: Field, active_or_nullified: bool) -> pub Field {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteViewerOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let opt_notes: [Option<ValueNote>; MAX_NOTES_PER_PAGE] = view_notes(storage_slot, options);

        opt_notes[0].unwrap().value
    }

    unconstrained fn call_view_notes_many(
        storage_slot: Field,
        active_or_nullified: bool
    ) -> pub [Field; 2] {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteViewerOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let opt_notes: [Option<ValueNote>; MAX_NOTES_PER_PAGE] = view_notes(storage_slot, options);

        [opt_notes[0].unwrap().value, opt_notes[1].unwrap().value]
    }

    #[aztec(private)]
    fn call_destroy_note(storage_slot: Field) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let options = NoteGetterOptions::new();
        let opt_notes: [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

        let note = opt_notes[0].unwrap();

        destroy_note(&mut context, note);
    }

    // Test codegen for Aztec.nr interfaces
    // See yarn-project/simulator/src/client/private_execution.test.ts 'nested calls through autogenerated interface'
    // Note; this function is deliberately NOT annotated with #[aztec(private)] due to its use in tests
    fn test_code_gen(
        inputs: PrivateContextInputs,
        a_field: Field,
        a_bool: bool,
        a_number: u32,
        an_array: [Field; 2],
        a_struct: DummyNote,
        a_deep_struct: DeepStruct
    ) -> distinct pub PrivateCircuitPublicInputs {
        let mut args = ArgsHasher::new();
        args.add(a_field);
        args.add(a_bool as Field);
        args.add(a_number as Field);
        args.add_multiple(an_array);
        args.add(a_struct.amount);
        args.add(a_struct.secret_hash);
        args.add(a_deep_struct.a_field);
        args.add(a_deep_struct.a_bool as Field);
        args.add(a_deep_struct.a_note.amount);
        args.add(a_deep_struct.a_note.secret_hash);
        for note in a_deep_struct.many_notes {
            args.add(note.amount);
            args.add(note.secret_hash);
        }
        let args_hash = args.hash();
        let mut context = PrivateContext::new(inputs, args_hash);
        context.return_values.push(args_hash);
        context.finish()
    }

    // Purely exists for testing
    #[aztec(public)]
    fn create_l2_to_l1_message_public(amount: Field, secret_hash: Field) {
        // Create a commitment to the amount
        let note = DummyNote::new(amount, secret_hash);

        // Public oracle call to emit new commitment.
        context.message_portal(context.this_portal_address(), note.get_commitment());
    }

    #[aztec(public)]
    fn create_l2_to_l1_message_arbitrary_recipient_public(content: Field, recipient: EthAddress) {
        // Public oracle call to emit new commitment.
        context.message_portal(recipient, content);
    }

    #[aztec(private)]
    fn create_l2_to_l1_message_arbitrary_recipient_private(content: Field, recipient: EthAddress) {
        // Public oracle call to emit new commitment.
        context.message_portal(recipient, content);
    }

    // Purely exists for testing
    #[aztec(public)]
    fn create_nullifier_public(amount: Field, secret_hash: Field) {
        // Create a commitment to the amount
        let note = DummyNote::new(amount, secret_hash);

        // Public oracle call to emit new commitment.
        context.push_new_nullifier(note.get_commitment(), 0);
    }

    // Forcefully emits a nullifier (for testing purposes)
    #[aztec(private)]
    fn emit_nullifier(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
    }

    #[aztec(private)]
    fn emit_msg_sender() {
        // Note: don't use emit_unencrypted_log_from_private in production code
        emit_unencrypted_log_from_private(&mut context, context.msg_sender());
    }

    #[aztec(private)]
    fn emit_array_as_unencrypted_log(fields: [Field; 5]) {
        emit_unencrypted_log_from_private(&mut context, fields);
    }

    // docs:start:is-time-equal
    #[aztec(public)]
    fn is_time_equal(time: u64) -> u64 {
        assert(context.timestamp() == time);
        time
    }
    // docs:end:is-time-equal

    #[aztec(public)]
    fn emit_unencrypted(value: Field) -> Field {
        // docs:start:emit_unencrypted
        emit_unencrypted_log(&mut context, value);
        // docs:end:emit_unencrypted
        0
    }

    #[aztec(public)]
    fn consume_mint_public_message(to: AztecAddress, amount: Field, secret: Field) {
        let content_hash = get_mint_public_content_hash(to, amount);
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, context.this_portal_address());
    }

    #[aztec(private)]
    fn consume_mint_private_message(
        secret_hash_for_redeeming_minted_notes: Field,
        amount: Field,
        secret_for_L1_to_L2_message_consumption: Field
    ) {
        // Consume L1 to L2 message and emit nullifier
        let content_hash = get_mint_private_content_hash(secret_hash_for_redeeming_minted_notes, amount);
        context.consume_l1_to_l2_message(
            content_hash,
            secret_for_L1_to_L2_message_consumption,
            context.this_portal_address()
        );
    }

    #[aztec(public)]
    fn consume_message_from_arbitrary_sender_public(
        content: Field,
        secret: Field,
        sender: EthAddress
    ) {
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content, secret, sender);
    }

    #[aztec(private)]
    fn consume_message_from_arbitrary_sender_private(
        content: Field,
        secret: Field,
        sender: EthAddress
    ) {
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content, secret, sender);
    }

    #[aztec(private)]
    fn set_constant(value: Field) {
        let mut note = FieldNote::new(value);
        storage.example_constant.initialize(&mut note, false);
    }

    #[aztec(private)]
    fn assert_private_global_vars(chain_id: Field, version: Field) {
        assert(context.chain_id() == chain_id, "Invalid chain id");
        assert(context.version() == version, "Invalid version");
    }

    #[aztec(public)]
    fn assert_public_global_vars(
        chain_id: Field,
        version: Field,
        block_number: Field,
        timestamp: u64,
        coinbase: EthAddress,
        fee_recipient: AztecAddress
    ) {
        assert(context.chain_id() == chain_id, "Invalid chain id");
        assert(context.version() == version, "Invalid version");
        assert(context.block_number() == block_number, "Invalid block number");
        assert(context.timestamp() == timestamp, "Invalid timestamp");
        assert(context.coinbase() == coinbase, "Invalid coinbase");
        assert(context.fee_recipient() == fee_recipient, "Invalid fee recipient");
    }

    #[aztec(private)]
    fn assert_header_private(header_hash: Field) {
        assert(context.historical_header.hash() == header_hash, "Invalid header hash");
    }

    #[aztec(public)]
    fn assert_header_public(header_hash: Field) {
        assert(context.historical_header.hash() == header_hash, "Invalid header hash");
    }

    #[aztec(private)]
    fn deploy_contract(target: AztecAddress) {
        aztec_deploy_contract(&mut context, target);
    }

    #[aztec(private)]
    // Adapted from TokenContract#redeem_shield but without an initcheck so it can be run in simulator/src/client/private_execution.test.ts
    fn consume_note_from_secret(secret: Field) {
        let notes_set = storage.example_set;
        let secret_hash = compute_secret_hash(secret);
        let options = NoteGetterOptions::new().select(FieldNote::properties().value, secret_hash, Option::none()).set_limit(1);
        let notes = notes_set.get_notes(options);
        let note = notes[0].unwrap_unchecked();
        notes_set.remove(note);
    }

    unconstrained fn get_constant() -> pub Field {
        let constant = storage.example_constant.view_note();
        constant.value
    }

    // Purely exists for testing
    unconstrained fn get_random(kinda_seed: Field) -> pub Field {
        kinda_seed * unsafe_rand()
    }

    struct DummyNote {
        amount: Field,
        secret_hash: Field
    }

    impl DummyNote {
        fn new(amount: Field, secret_hash: Field) -> Self {
            Self { amount, secret_hash }
        }

        fn get_commitment(self) -> Field {
            pedersen_hash([self.amount, self.secret_hash], 0)
        }
    }

    struct DeepStruct {
        a_field: Field,
        a_bool: bool,
        a_note: DummyNote,
        many_notes: [DummyNote; 3],
    }
}
