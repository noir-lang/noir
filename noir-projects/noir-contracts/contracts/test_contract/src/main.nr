mod test_note;

// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract Test {

    use dep::aztec::prelude::{
        AztecAddress, EthAddress, FunctionSelector, NoteHeader, NoteGetterOptions, NoteViewerOptions,
        PrivateContext, PrivateImmutable, PrivateSet, SharedImmutable
    };
    use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note;
    use dep::aztec::encrypted_logs::encrypted_event_emission::encode_and_encrypt_event_with_keys_with_randomness;

    use dep::aztec::protocol_types::{
        abis::private_circuit_public_inputs::PrivateCircuitPublicInputs,
        constants::{MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, CANONICAL_KEY_REGISTRY_ADDRESS},
        traits::{Serialize, ToField, FromField}, point::Point, scalar::Scalar,
        storage::map::derive_storage_slot_in_map
    };

    use dep::aztec::encrypted_logs::header::EncryptedLogHeader;
    use dep::aztec::encrypted_logs::incoming_body::EncryptedLogIncomingBody;
    use dep::aztec::encrypted_logs::outgoing_body::EncryptedLogOutgoingBody;

    use dep::aztec::note::constants::MAX_NOTES_PER_PAGE;

    use dep::aztec::state_vars::{shared_mutable::SharedMutablePrivateGetter};

    use dep::aztec::{
        context::inputs::private_context_inputs::PrivateContextInputs,
        hash::{pedersen_hash, compute_secret_hash, ArgsHasher},
        note::{
        lifecycle::{create_note, destroy_note}, note_getter::{get_notes, view_notes},
        note_getter_options::NoteStatus
    },
        deploy::deploy_contract as aztec_deploy_contract,
        oracle::{encryption::aes128_encrypt, unsafe_rand::unsafe_rand}
    };
    use dep::token_portal_content_hash_lib::{get_mint_private_content_hash, get_mint_public_content_hash};
    use dep::value_note::value_note::ValueNote;
    use std::embedded_curve_ops::fixed_base_scalar_mul as derive_public_key;

    use crate::test_note::TestNote;

    #[aztec(event)]
    struct ExampleEvent {
        value0: Field,
        value1: Field,
        value2: Field,
        value3: Field,
        value4: Field,
    }

    #[aztec(storage)]
    struct Storage {
        example_constant: PrivateImmutable<TestNote>,
        example_set: PrivateSet<TestNote>,
    }

    #[aztec(private)]
    fn get_ovsk_app(ovpk_m_hash: Field) -> Field {
        context.request_ovsk_app(ovpk_m_hash)
    }

    #[aztec(private)]
    fn get_master_incoming_viewing_public_key(address: AztecAddress) -> [Field; 2] {
        let header = context.get_header();
        let pub_key = header.get_ivpk_m(&mut context, address);

        [pub_key.x, pub_key.y]
    }

    // Get the address of this contract (taken from the input context)
    #[aztec(private)]
    fn get_this_address() -> AztecAddress {
        context.this_address()
    }

    #[aztec(private)]
    fn set_tx_max_block_number(max_block_number: u32, enqueue_public_call: bool) {
        context.set_tx_max_block_number(max_block_number);

        if enqueue_public_call {
            Test::at(context.this_address()).dummy_public_call().enqueue(&mut context)
        }
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn dummy_public_call() {}

    #[aztec(private)]
    fn call_create_note(
        value: Field,
        owner: AztecAddress,
        outgoing_viewer: AztecAddress,
        storage_slot: Field
    ) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);

        let mut note = ValueNote::new(value, owner_npk_m_hash);
        create_note(&mut context, storage_slot, &mut note).emit(encode_and_encrypt_note(&mut context, outgoing_viewer, owner));
    }

    #[aztec(private)]
    fn call_get_notes(storage_slot: Field, active_or_nullified: bool) -> Field {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteGetterOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let notes: BoundedVec<ValueNote, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL> = get_notes(&mut context, storage_slot, options);

        notes.get(0).value
    }

    #[aztec(private)]
    fn call_get_notes_many(storage_slot: Field, active_or_nullified: bool) -> [Field; 2] {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteGetterOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let notes: BoundedVec<ValueNote, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL> = get_notes(&mut context, storage_slot, options);

        [notes.get(0).value, notes.get(1).value]
    }

    unconstrained fn call_view_notes(storage_slot: Field, active_or_nullified: bool) -> pub Field {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteViewerOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let notes: BoundedVec<ValueNote, MAX_NOTES_PER_PAGE> = view_notes(storage_slot, options);

        notes.get(0).value
    }

    unconstrained fn call_view_notes_many(storage_slot: Field, active_or_nullified: bool) -> pub [Field; 2] {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let mut options = NoteViewerOptions::new();
        if (active_or_nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }

        let notes: BoundedVec<ValueNote, MAX_NOTES_PER_PAGE> = view_notes(storage_slot, options);

        [notes.get(0).value, notes.get(1).value]
    }

    #[aztec(private)]
    fn call_destroy_note(storage_slot: Field) {
        assert(
            storage_slot != storage.example_constant.get_storage_slot(), "this storage slot is reserved for example_constant"
        );

        let options = NoteGetterOptions::new();
        let notes: BoundedVec<ValueNote, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL> = get_notes(&mut context, storage_slot, options);

        let note = notes.get(0);

        destroy_note(&mut context, note);
    }

    #[aztec(private)]
    fn test_code_gen(
        a_field: Field,
        a_bool: bool,
        a_number: u32,
        an_array: [Field; 2],
        a_struct: DummyNote,
        a_deep_struct: DeepStruct
    ) -> Field {
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
        args.hash()
    }

    #[aztec(private)]
    fn test_setting_teardown() {
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("dummy_public_call()"),
            []
        );
    }

    #[aztec(private)]
    fn test_setting_fee_payer() {
        context.set_as_fee_payer();
    }

    // Purely exists for testing
    #[aztec(public)]
    fn create_l2_to_l1_message_public(amount: Field, secret_hash: Field, portal_address: EthAddress) {
        // Create a commitment to the amount
        let note = DummyNote::new(amount, secret_hash);

        // Public oracle call to emit new commitment.
        context.message_portal(portal_address, note.get_commitment());
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
    fn emit_nullifier_public(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
    }

    // Forcefully emits a nullifier (for testing purposes)
    #[aztec(private)]
    fn emit_nullifier(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
    }

    // For testing non-note encrypted logs
    #[aztec(private)]
    fn emit_array_as_encrypted_log(
        fields: [Field; 5],
        owner: AztecAddress,
        outgoing_viewer: AztecAddress,
        nest: bool
    ) {
        let header = context.get_header();
        let outgoing_viewer_ovpk_m = header.get_ovpk_m(&mut context, outgoing_viewer);
        let owner_ivpk_m = header.get_ivpk_m(&mut context, owner);

        let event = ExampleEvent { value0: fields[0], value1: fields[1], value2: fields[2], value3: fields[3], value4: fields[4] };

        event.emit(
            encode_and_encrypt_event_with_keys_with_randomness(
                &mut context,
                // testing only - a secret random value is passed in here to salt / mask the address
                5,
                outgoing_viewer_ovpk_m,
                owner_ivpk_m
            )
        );

        // this contract has reached max number of functions, so using this one fn
        // to test nested and non nested encrypted logs
        if nest {
            Test::at(context.this_address()).emit_array_as_encrypted_log([0, 0, 0, 0, 0], owner, outgoing_viewer, false).call(&mut context);

            let otherEvent = ExampleEvent { value0: 1, value1: 2, value2: 3, value3: 4, value4: 5 };

            otherEvent.emit(
                encode_and_encrypt_event_with_keys_with_randomness(
                    &mut context,
                    // testing only - a randomness of 0 signals the kerels to not mask the address
                    0,
                    outgoing_viewer_ovpk_m,
                    owner_ivpk_m
                )
            );
        }
    }

    #[aztec(private)]
    fn emit_encrypted_logs_nested(value: Field, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let mut storage_slot = storage.example_constant.get_storage_slot() + 1;
        Test::at(context.this_address()).call_create_note(value, owner, outgoing_viewer, storage_slot).call(&mut context);
        storage_slot += 1;

        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);

        let mut note = ValueNote::new(value + 1, owner_npk_m_hash);
        create_note(&mut context, storage_slot, &mut note).emit(encode_and_encrypt_note(&mut context, context.msg_sender(), owner));
        storage_slot += 1;
        Test::at(context.this_address()).call_create_note(value + 2, owner, outgoing_viewer, storage_slot).call(&mut context);
    }

    // docs:start:is-time-equal
    #[aztec(public)]
    fn is_time_equal(time: u64) -> u64 {
        assert(context.timestamp() == time);
        time
    }
    // docs:end:is-time-equal

    #[aztec(public)]
    fn emit_unencrypted(value: Field) {
        // docs:start:emit_unencrypted
        context.emit_unencrypted_log(/*message=*/ value);
        context.emit_unencrypted_log(/*message=*/ [10, 20, 30]);
        context.emit_unencrypted_log(/*message=*/ "Hello, world!");
        // docs:end:emit_unencrypted
    }

    #[aztec(public)]
    fn consume_mint_public_message(
        to: AztecAddress,
        amount: Field,
        secret: Field,
        message_leaf_index: Field,
        portal_address: EthAddress
    ) {
        let content_hash = get_mint_public_content_hash(to, amount);
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, portal_address, message_leaf_index);
    }

    #[aztec(private)]
    fn consume_mint_private_message(
        secret_hash_for_redeeming_minted_notes: Field,
        amount: Field,
        secret_for_L1_to_L2_message_consumption: Field,
        portal_address: EthAddress
    ) {
        // Consume L1 to L2 message and emit nullifier
        let content_hash = get_mint_private_content_hash(secret_hash_for_redeeming_minted_notes, amount);
        context.consume_l1_to_l2_message(
            content_hash,
            secret_for_L1_to_L2_message_consumption,
            portal_address
        );
    }

    #[aztec(public)]
    fn consume_message_from_arbitrary_sender_public(
        content: Field,
        secret: Field,
        sender: EthAddress,
        message_leaf_index: Field
    ) {
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content, secret, sender, message_leaf_index);
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
        let mut note = TestNote::new(value);
        storage.example_constant.initialize(&mut note).discard();
    }

    #[aztec(private)]
    fn assert_private_global_vars(chain_id: Field, version: Field) {
        assert(context.chain_id() == chain_id, "Invalid chain id");
        assert(context.version() == version, "Invalid version");
    }

    #[aztec(private)]
    fn encrypt(input: [u8; 64], iv: [u8; 16], key: [u8; 16]) -> [u8; 80] {
        aes128_encrypt(input, iv, key)
    }

    #[aztec(private)]
    fn encrypt_with_padding(input: [u8; 65], iv: [u8; 16], key: [u8; 16]) -> [u8; 80] {
        aes128_encrypt(input, iv, key)
    }

    #[aztec(private)]
    fn compute_note_header_ciphertext(secret: Scalar, point: Point) -> [u8; 48] {
        EncryptedLogHeader::new(context.this_address()).compute_ciphertext(secret, point)
    }

    // 64 bytes + 32 * #fields + 16 = 112 bytes
    #[aztec(private)]
    fn compute_incoming_log_body_ciphertext(
        secret: Scalar,
        point: Point,
        storage_slot: Field,
        value: Field
    ) -> [u8; 112] {
        let note = TestNote::new(value);
        EncryptedLogIncomingBody::from_note(note, storage_slot).compute_ciphertext(secret, point).as_array()
    }

    #[aztec(private)]
    fn compute_outgoing_log_body_ciphertext(
        eph_sk: Scalar,
        recipient: AztecAddress,
        recipient_ivpk_app: Point,
        ovsk_app: Scalar
    ) -> [u8; 176] {
        let eph_pk = derive_public_key(eph_sk);
        EncryptedLogOutgoingBody::new(eph_sk, recipient, recipient_ivpk_app).compute_ciphertext(ovsk_app, eph_pk)
    }

    #[aztec(public)]
    fn assert_public_global_vars(
        chain_id: Field,
        version: Field,
        block_number: Field,
        timestamp: u64,
        fee_per_da_gas: Field,
        fee_per_l2_gas: Field
    ) {
        assert(context.chain_id() == chain_id, "Invalid chain id");
        assert(context.version() == version, "Invalid version");
        assert(context.block_number() == block_number, "Invalid block number");
        assert(context.timestamp() == timestamp, "Invalid timestamp");
        assert(context.fee_per_da_gas() == fee_per_da_gas, "Invalid fee per da gas");
        assert(context.fee_per_l2_gas() == fee_per_l2_gas, "Invalid fee per l2 gas");
    }

    #[aztec(private)]
    fn assert_header_private(header_hash: Field) {
        assert(context.historical_header.hash() == header_hash, "Invalid header hash");
    }

    // TODO(4840): add AVM opcodes for getting header (members)
    //#[aztec(public)]
    //fn assert_header_public(header_hash: Field) {
    //    assert(context.historical_header.hash() == header_hash, "Invalid header hash");
    //}

    #[aztec(private)]
    fn deploy_contract(target: AztecAddress) {
        aztec_deploy_contract(&mut context, target);
    }

    #[aztec(private)]
    // Adapted from TokenContract#redeem_shield but without an initcheck so it can be run in simulator/src/client/private_execution.test.ts
    fn consume_note_from_secret(secret: Field) {
        let notes_set = storage.example_set;
        let secret_hash = compute_secret_hash(secret);
        let mut options = NoteGetterOptions::new();
        options = options.select(TestNote::properties().value, secret_hash, Option::none()).set_limit(1);
        let notes = notes_set.get_notes(options);
        let note = notes.get_unchecked(0);
        notes_set.remove(note);
    }

    unconstrained fn get_constant() -> pub Field {
        let constant = storage.example_constant.view_note();
        constant.value
    }

    // This function is used in the e2e_state_vars to test the SharedMutablePrivateGetter in isolation
    #[aztec(private)]
    fn test_shared_mutable_private_getter(
        contract_address_to_read: AztecAddress,
        storage_slot_of_shared_mutable: Field
    ) -> Field {
        // It's a bit wonky because we need to know the delay for get_current_value_in_private to work correctly
        let test: SharedMutablePrivateGetter<AztecAddress, 5> = SharedMutablePrivateGetter::new(
            &mut context,
            contract_address_to_read,
            storage_slot_of_shared_mutable
        );

        let ret = test.get_value_in_private(context.get_header());

        ret.to_field()
    }

    // This function is used for testing the registry contract and fresh public key getters. If nothing exists in the registry, but we have added public
    // keys to the pxe, this function will return nothing, but the public key getters will return the correct value
    #[aztec(private)]
    fn test_shared_mutable_private_getter_for_registry_contract(
        storage_slot_of_shared_mutable: Field,
        address_to_get_in_registry: AztecAddress
    ) -> Field {
        // We have to derive this slot to get the location of the shared mutable inside the Map
        let derived_slot = derive_storage_slot_in_map(storage_slot_of_shared_mutable, address_to_get_in_registry);

        // It's a bit wonky because we need to know the delay for get_current_value_in_private to work correctly
        let registry_private_getter: SharedMutablePrivateGetter<Field, 5> = SharedMutablePrivateGetter::new(
            &mut context,
            AztecAddress::from_field(CANONICAL_KEY_REGISTRY_ADDRESS),
            derived_slot
        );
        registry_private_getter.get_value_in_private(context.get_header())
    }

    #[aztec(private)]
    fn test_nullifier_key_freshness(address: AztecAddress, public_nullifying_key: Point) {
        assert_eq(context.get_header().get_npk_m(&mut context, address), public_nullifying_key);
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

    impl Serialize<2> for DummyNote {
        fn serialize(self) -> [Field; 2] {
            [self.amount, self.secret_hash]
        }
    }

    struct DeepStruct {
        a_field: Field,
        a_bool: bool,
        a_note: DummyNote,
        many_notes: [DummyNote; 3],
    }

    // Serializing using "canonical" form. 
    // 1. Everything that fits in a field, *becomes* a Field
    // 2. Strings become arrays of bytes (no strings here)
    // 4. Arrays become arrays of Fields following rules 2 and 3 (no arrays here)
    // 5. Structs become arrays of Fields, with every item defined in the same order as they are in Noir code, following rules 2, 3, 4 and 5 (recursive)
    impl Serialize<10> for DeepStruct {
        fn serialize(self) -> [Field; 10] {
            let mut result = [0; 10];
            result[0] = self.a_field;
            result[1] = self.a_bool as Field;
            result[2] = self.a_note.amount;
            result[3] = self.a_note.secret_hash;
            for i in 0..3 {
                result[4 + i * 2] = self.many_notes[i].amount;
                result[5 + i * 2] = self.many_notes[i].secret_hash;
            }
            result
        }
    }
}
