// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::aztec::prelude::{
        AztecAddress, EthAddress, FunctionSelector, NoteHeader, NoteGetterOptions, PrivateContext, Map,
        PrivateSet, PublicMutable
    };
    use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note;

    use dep::aztec::protocol_types::{contract_class_id::ContractClassId, header::Header};
    use dep::aztec::{note::note_getter_options::NoteStatus};
    // docs:start:imports
    // Imports are not needed as inclusion / non_inclusion proofs are accessible on the header.
    // docs:end:imports
    // docs:start:value_note_imports
    use dep::value_note::value_note::ValueNote;
    // docs:end:value_note_imports
    #[aztec(storage)]
    struct Storage {
        private_values: Map<AztecAddress, PrivateSet<ValueNote>>,
        public_value: PublicMutable<Field>,
        public_unused_value: PublicMutable<Field>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(public_value: Field) {
        storage.public_value.write(public_value);
    }

    // docs:start:create_note
    // Creates a value note owned by `owner`.
    #[aztec(private)]
    fn create_note(owner: AztecAddress, value: Field) {
        let owner_private_values = storage.private_values.at(owner);
        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);

        let mut note = ValueNote::new(value, owner_npk_m_hash);
        owner_private_values.insert(&mut note).emit(encode_and_encrypt_note(&mut context, context.msg_sender(), owner));
    }
    // docs:end:create_note

    #[aztec(private)]
    fn test_note_inclusion(
        owner: AztecAddress,
        use_block_number: bool,
        block_number: u32, // The block at which we'll prove that the note exists
        nullified: bool
    ) {
        // docs:start:get_note_from_pxe
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.set_limit(1);
        if (nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let note = private_values.get_notes(options).get_unchecked(0);
        // docs:end:get_note_from_pxe

        // 2) Prove the note inclusion
        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };
        // docs:start:prove_note_inclusion
        header.prove_note_inclusion(note);
        // docs:end:prove_note_inclusion
    }

    #[aztec(private)]
    fn test_note_inclusion_fail_case(
        owner: AztecAddress,
        use_block_number: bool,
        block_number: u32 // The block at which we'll prove that the note exists
    ) {
        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);
        let mut note = ValueNote::new(1, owner_npk_m_hash);

        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };

        header.prove_note_inclusion(note);
    }

    // Proves that the note was not yet nullified at block `block_number`.
    #[aztec(private)]
    fn test_note_not_nullified(
        owner: AztecAddress,
        use_block_number: bool,
        block_number: u32, // The block at which we'll prove that the nullifier does not exists
        // Value below is only used when the note is not found --> used to test the nullifier non-inclusion failure
        // case (it allows me to pass in random value of note nullifier - I cannot add and fetch a random note from PXE
        // because PXE performs note commitment inclusion check when you add a new note).
        fail_case: bool
    ) {
        // 1) Get the note from PXE
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.set_limit(1);
        if (fail_case) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let note = private_values.get_notes(options).get_unchecked(0);

        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };
        // docs:start:prove_note_not_nullified
        header.prove_note_not_nullified(note, &mut context);
        // docs:end:prove_note_not_nullified
    }

    #[aztec(private)]
    fn test_note_validity(
        owner: AztecAddress,
        use_block_number: bool,
        block_number: u32, // The block at which we'll prove that the note exists and is not nullified
        nullified: bool
    ) {
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.set_limit(1);
        if (nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let note = private_values.get_notes(options).get(0);

        // 2) Prove the note validity
        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };
        // docs:start:prove_note_validity
        header.prove_note_validity(note, &mut context);
        // docs:end:prove_note_validity
    }

    // docs:start:nullify_note
    #[aztec(private)]
    fn nullify_note(owner: AztecAddress) {
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes.get(0);

        private_values.remove(note);
    }
    // docs:end:nullify_note

    // Proves nullifier existed at block `block_number`.
    // Note: I am not getting a nullifier of the note that was created in this contract in this function because it is
    //       currently not possible to obtain a nullified note from PXE.
    #[aztec(private)]
    fn test_nullifier_inclusion(
        nullifier: Field,
        use_block_number: bool,
        block_number: u32 // The block at which we'll prove that the nullifier exists in the nullifier tree
    ) {
        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };
        // docs:start:prove_nullifier_inclusion
        header.prove_nullifier_inclusion(nullifier);
        // docs:end:prove_nullifier_inclusion
    }

    #[aztec(public)]
    fn push_nullifier_public(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
    }

    // Proves nullifier existed at latest block
    #[aztec(public)]
    fn test_nullifier_inclusion_from_public(nullifier: Field) {
        assert(context.nullifier_exists(nullifier, context.this_address()));
    }

    #[aztec(private)]
    fn test_storage_historical_read_unset_slot(block_number: u32 // The block at which we'll read the public storage value
    ) {
        let header = context.get_header_at(block_number);

        assert_eq(
            // docs:start:public_storage_historical_read
            header.public_storage_historical_read(
                storage.public_unused_value.storage_slot,
                context.this_address()
            // docs:end:public_storage_historical_read
            ), 0
        );
    }

    #[aztec(private)]
    fn test_storage_historical_read(
        expected: Field,
        use_block_number: bool,
        block_number: u32 // The block at which we'll read the public storage value
    ) {
        let header = if (use_block_number) {
            context.get_header_at(block_number)
        } else {
            context.get_header()
        };

        let actual = header.public_storage_historical_read(storage.public_value.storage_slot, context.this_address());

        assert_eq(actual, expected, "Actual public value does not match expected");
    }

    // Proves that a contract was publicly deployed and/or initialized at block `block_number`.
    #[aztec(private)]
    fn test_contract_inclusion(
        contract_address: AztecAddress,
        block_number: u32,
        test_deployment: bool,
        test_initialization: bool
    ) {
        let header = context.get_header_at(block_number);

        if test_deployment {
            // docs:start:prove_contract_deployment
            header.prove_contract_deployment(contract_address);
            // docs:end:prove_contract_deployment
        }
        if test_initialization {
            // docs:start:prove_contract_initialization
            header.prove_contract_initialization(contract_address);
            // docs:end:prove_contract_initialization
        }
    }

    // Proves that a contract was NOT publicly deployed and/or initialized at block `block_number`.
    #[aztec(private)]
    fn test_contract_non_inclusion(
        contract_address: AztecAddress,
        block_number: u32,
        test_deployment: bool,
        test_initialization: bool
    ) {
        let header = context.get_header_at(block_number);

        if test_deployment {
            // docs:start:prove_contract_non_deployment
            header.prove_contract_non_deployment(contract_address);
            // docs:end:prove_contract_non_deployment
        }
        if test_initialization {
            // docs:start:prove_contract_non_initialization
            header.prove_contract_non_initialization(contract_address);
            // docs:end:prove_contract_non_initialization
        }
    }
}
