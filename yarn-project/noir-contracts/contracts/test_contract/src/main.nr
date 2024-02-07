// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract Test {
    use dep::std::option::Option;
    use dep::aztec::protocol_types::{
        address::{
            AztecAddress,
            EthAddress,
        },
        constants::{
            MAX_READ_REQUESTS_PER_CALL,
            MAX_NOTES_PER_PAGE
        }
    };
    // The following import is here in order to make the event macro work because the macro doesn't add the import.
    // It doesn't add the import because in the future we will re-export all the types via aztec-nr and aztec-nr is
    // already auto-imported by the macros.
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3590): Remove this once the issue is fixed.
    use dep::aztec::protocol_types;
    // docs:start:unencrypted_import
    use dep::aztec::log::emit_unencrypted_log;
    // docs:end:unencrypted_import

    use dep::aztec::{
        context::Context,
        abi,
        abi::PrivateContextInputs,
        hash::pedersen_hash,
        context::PrivateContext,
        note::{
            note_header::NoteHeader,
            utils as note_utils,
            lifecycle::{create_note, destroy_note},
            note_getter::{get_notes, view_notes},
            note_getter_options::{NoteGetterOptions, NoteStatus},
            note_viewer_options::NoteViewerOptions,
        },
        oracle::{
            get_public_key::get_public_key as get_public_key_oracle,
            context::get_portal_address,
            rand::rand
        },
        state_vars::immutable_singleton::ImmutableSingleton,
        log::emit_unencrypted_log_from_private,
    };
    use dep::token_portal_content_hash_lib::{get_mint_private_content_hash, get_mint_public_content_hash};
    use dep::field_note::field_note::FieldNote;
    use dep::value_note::value_note::{ValueNote,VALUE_NOTE_LEN};

    #[event]
    struct ExampleEvent {
        value: Field,
    }

    struct Storage {
        example_constant: ImmutableSingleton<FieldNote>,
    }

    #[aztec(private)]
    // docs:start:empty-constructor
    fn constructor() {}
    // docs:end:empty-constructor

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

        let opt_notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

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

        let opt_notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

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
        let opt_notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] = get_notes(&mut context, storage_slot, options);

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
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut args: BoundedVec<Field, 17> = BoundedVec::new(0);
        args.push(a_field);
        args.push(a_bool as Field);
        args.push(a_number as Field);
        args.extend_from_array(an_array);
        args.push(a_struct.amount);
        args.push(a_struct.secret_hash);
        args.push(a_deep_struct.a_field);
        args.push(a_deep_struct.a_bool as Field);
        args.push(a_deep_struct.a_note.amount);
        args.push(a_deep_struct.a_note.secret_hash);
        for note in a_deep_struct.many_notes {
            args.push(note.amount);
            args.push(note.secret_hash);
        }
        let args_hash = abi::hash_args(args.storage);
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
        context.message_portal(note.get_commitment());
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
    fn is_time_equal(time: Field) -> Field {
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
    fn consume_mint_public_message(
        to: AztecAddress,
        amount: Field,
        canceller: EthAddress,
        msg_key: Field,
        secret: Field
    ) {
        let content_hash = get_mint_public_content_hash(to, amount, canceller);
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret);
    }

    #[aztec(private)]
    fn consume_mint_private_message(
        secret_hash_for_redeeming_minted_notes: Field,
        amount: Field,
        canceller: EthAddress,
        msg_key: Field,
        secret_for_L1_to_L2_message_consumption: Field
    ) {
        // Consume L1 to L2 message and emit nullifier
        let content_hash = get_mint_private_content_hash(secret_hash_for_redeeming_minted_notes, amount, canceller);
        context.consume_l1_to_l2_message(msg_key, content_hash, secret_for_L1_to_L2_message_consumption);
    }

    #[aztec(private)]
    fn set_constant(value: Field) {
        let mut note = FieldNote::new(value);
        storage.example_constant.initialize(&mut note, false);
    }

    #[aztec(public)]
    fn assert_coinbase(coinbase: EthAddress) {
        assert(context.coinbase().eq(coinbase), "Invalid coinbase");
    }

    #[aztec(public)]
    fn assert_fee_recipient(fee_recipient: AztecAddress) {
        assert(context.fee_recipient().eq(fee_recipient), "Invalid fee recipient");
    }

    unconstrained fn get_constant() -> pub Field {
        let constant = storage.example_constant.view_note();
        constant.value
    }

    // Purely exists for testing
    unconstrained fn get_random(kindaSeed: Field) -> pub Field {
        kindaSeed * rand()
    }

    struct DummyNote {
        amount: Field,
        secret_hash: Field
    }

    impl DummyNote {
        fn new(amount: Field, secret_hash: Field) -> Self {
            Self {
                amount: amount,
                secret_hash: secret_hash
            }
        }

        fn get_commitment(self) -> Field {
            pedersen_hash([self.amount, self.secret_hash],0)
        }
    }

    struct DeepStruct {
        a_field: Field,
        a_bool: bool,
        a_note: DummyNote,
        many_notes: [DummyNote; 3],
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN] // must fit either a FieldNote or a ValueNote
    ) -> pub [Field; 4] {
        if (storage_slot == storage.example_constant.get_storage_slot()) {
            let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
            note_utils::compute_note_hash_and_nullifier(FieldNote::deserialize_content, note_header, serialized_note)
        } else {
            // For ValueNotes created via write_value_to_storage
            let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
            note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
        }
    }
}
