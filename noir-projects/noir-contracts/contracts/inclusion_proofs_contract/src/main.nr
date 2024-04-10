// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::aztec::prelude::{
        AztecAddress, EthAddress, FunctionSelector, NoteHeader, NoteGetterOptions, PrivateContext, Map,
        PrivateSet, PublicMutable
    };

    use dep::aztec::protocol_types::{grumpkin_point::GrumpkinPoint, contract_class_id::ContractClassId};
    use dep::aztec::{context::Context, note::note_getter_options::NoteStatus};
    // docs:start:imports
    use dep::aztec::history::{
        contract_inclusion::{
        prove_contract_deployment_at, prove_contract_non_deployment_at,
        prove_contract_initialization_at, prove_contract_non_initialization_at
    },
        note_inclusion::{prove_note_inclusion, prove_note_inclusion_at},
        note_validity::{prove_note_validity, prove_note_validity_at},
        nullifier_inclusion::{
        prove_nullifier_inclusion, prove_nullifier_inclusion_at, prove_note_is_nullified,
        prove_note_is_nullified_at
    },
        nullifier_non_inclusion::{prove_note_not_nullified, prove_note_not_nullified_at},
        public_storage::{public_storage_historical_read, public_storage_historical_read_at}
    };
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
        let mut note = ValueNote::new(value, owner);
        owner_private_values.insert(&mut note, true);
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
        options = options.select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        if (nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];
        // docs:end:get_note_from_pxe

        // 2) Prove the note inclusion
        if (use_block_number) {
            // docs:start:prove_note_inclusion
            prove_note_inclusion_at(maybe_note.unwrap_unchecked(), block_number, context);
            // docs:end:prove_note_inclusion
        } else {
            prove_note_inclusion(maybe_note.unwrap_unchecked(), context);
        }
    }

    #[aztec(private)]
    fn test_note_inclusion_fail_case(
        owner: AztecAddress,
        use_block_number: bool,
        block_number: u32 // The block at which we'll prove that the note exists
    ) {
        let mut note = ValueNote::new(1, owner);

        if (use_block_number) {
            prove_note_inclusion_at(note, block_number, context);
        } else {
            prove_note_inclusion(note, context);
        }
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
        // 2) Get the note from PXE
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        if (fail_case) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 3) Compute the nullifier from the note
        // docs:start:prove_note_not_nullified
        if (use_block_number) {
            prove_note_not_nullified_at(maybe_note.unwrap_unchecked(), block_number, &mut context);
        } else {
            prove_note_not_nullified(maybe_note.unwrap_unchecked(), &mut context);
        }
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
        options = options.select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        if (nullified) {
            options = options.set_status(NoteStatus.ACTIVE_OR_NULLIFIED);
        }
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

        // 2) Prove the note validity
        if (use_block_number) {
            // docs:start:prove_note_validity
            prove_note_validity_at(note, block_number, &mut context);
            // docs:end:prove_note_validity
        } else {
            prove_note_validity(note, &mut context);
        }
    }

    // docs:start:nullify_note
    #[aztec(private)]
    fn nullify_note(owner: AztecAddress) {
        let private_values = storage.private_values.at(owner);
        let mut options = NoteGetterOptions::new();
        options = options.select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

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
        if (use_block_number) {
            // docs:start:prove_nullifier_inclusion
            prove_nullifier_inclusion_at(nullifier, block_number, context);
            // docs:end:prove_nullifier_inclusion
        } else {
            prove_nullifier_inclusion(nullifier, context);
        }
    }

    // Proves nullifier existed at latest block
    #[aztec(public)]
    fn test_nullifier_inclusion_from_public(nullifier: Field) {
        assert(context.nullifier_exists(nullifier));
    }

    #[aztec(private)]
    fn test_storage_historical_read_unset_slot(block_number: u32 // The block at which we'll read the public storage value
    ) {
        assert_eq(
            public_storage_historical_read_at(
                context,
                storage.public_unused_value.storage_slot,
                context.this_address(),
                block_number
            ), 0
        );
    }

    #[aztec(private)]
    fn test_storage_historical_read(
        expected: Field,
        use_block_number: bool,
        block_number: u32 // The block at which we'll read the public storage value
    ) {
        let actual = if (use_block_number) {
            public_storage_historical_read_at(
                context,
                storage.public_value.storage_slot,
                context.this_address(),
                block_number
            )
        } else {
            public_storage_historical_read(
                context,
                storage.public_value.storage_slot,
                context.this_address()
            )
        };

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
        if test_deployment {
            prove_contract_deployment_at(contract_address, block_number, context);
        }
        if test_initialization {
            prove_contract_initialization_at(contract_address, block_number, context);
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
        if test_deployment {
            prove_contract_non_deployment_at(contract_address, block_number, context);
        }
        if test_initialization {
            prove_contract_non_initialization_at(contract_address, block_number, context);
        }
    }
}
