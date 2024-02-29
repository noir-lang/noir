// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector, address::{AztecAddress, EthAddress},
        grumpkin_point::GrumpkinPoint, contract_class_id::ContractClassId
    };
    use dep::aztec::{
        state_vars::{Map, PrivateSet, PublicMutable}, context::Context,
        note::{
        note_getter_options::NoteGetterOptions, note_getter_options::NoteStatus,
        note_header::NoteHeader, utils as note_utils
    }
    };
    // docs:start:imports
    use dep::aztec::history::{
        contract_inclusion::{prove_contract_inclusion, prove_contract_inclusion_at},
        note_inclusion::{prove_note_inclusion, prove_note_inclusion_at},
        note_validity::{prove_note_validity, prove_note_validity_at},
        nullifier_inclusion::{
        prove_nullifier_inclusion, prove_nullifier_inclusion_at, prove_note_is_nullified,
        prove_note_is_nullified_at
    },
        nullifier_non_inclusion::{prove_note_not_nullified, prove_note_not_nullified_at},
        public_value_inclusion::{prove_public_value_inclusion, prove_public_value_inclusion_at}
    };
    // docs:end:imports
    // docs:start:value_note_imports
    use dep::value_note::value_note::ValueNote;
    // docs:end:value_note_imports
    struct Storage {
        private_values: Map<AztecAddress, PrivateSet<ValueNote>>,
        public_value: PublicMutable<Field>,
        public_unused_value: PublicMutable<Field>,
    }

    #[aztec(private)]
    fn constructor(public_value: Field) {
        let selector = FunctionSelector::from_signature("_initialize(Field)");
        context.call_public_function(context.this_address(), selector, [public_value]);
    }

    #[aztec(public)]
    internal fn _initialize(value: Field) {
        storage.public_value.write(value);
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
        let mut options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
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
        let mut options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
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
        let mut options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
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
        let options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
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

    #[aztec(private)]
    fn test_public_unused_value_inclusion(block_number: u32 // The block at which we'll prove that the public value exists
    ) {
        prove_public_value_inclusion_at(
            0,
            storage.public_unused_value.storage_slot,
            context.this_address(),
            block_number,
            context
        );
    }

    #[aztec(private)]
    fn test_public_value_inclusion(
        public_value: Field,
        use_block_number: bool,
        block_number: u32 // The block at which we'll prove that the public value exists
    ) {
        if (use_block_number) {
            prove_public_value_inclusion_at(
                public_value,
                storage.public_value.storage_slot,
                context.this_address(),
                block_number,
                context
            );
        } else {
            prove_public_value_inclusion(
                public_value,
                storage.public_value.storage_slot,
                context.this_address(),
                context
            );
        }
    }

    // Proves that a contract exists at block `block_number`.
    // Note: This can be used to approximate a factory pattern --> a factory contract could perform this proof and that
    //       way verify that a contract at a given address is what it expects. Then it could store it in an internal
    //       map of contracts (like what Uniswap Factory does with pool contracts - it stores them in a mapping).
    //       By passing in the construct hash the factory can also verify that the contract was constructed with the
    //       correct constructor arguments. Typically the factory would store the expected construct hash and assert
    //       that it is what it expects. The constructor param check is the reason of why we pass in the preimage of
    //       contract's aztec address instead of just the address.
    #[aztec(private)]
    fn test_contract_inclusion(
        public_key: GrumpkinPoint,
        contract_address_salt: Field,
        contract_class_id: ContractClassId,
        initialization_hash: Field,
        portal_contract_address: EthAddress,
        block_number: u32 // The block at which we'll prove that the public value exists
    ) {
        let proven_contract_address = prove_contract_inclusion_at(
            public_key,
            contract_address_salt,
            contract_class_id,
            initialization_hash,
            portal_contract_address,
            block_number,
            context
        );
        // Here typically the factory would add the contract address to its internal map of deployed contracts.
    }
}
