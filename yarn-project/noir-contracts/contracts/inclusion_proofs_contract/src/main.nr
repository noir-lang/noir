// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::{
            AztecAddress,
            EthAddress,
        },
        grumpkin_point::GrumpkinPoint,
    };
    use dep::aztec::{
        state_vars::{
            map::Map,
            set::Set,
            public_state::PublicState,
        },
        types::{
            type_serialization::field_serialization::FieldSerializationMethods,
        },
        context::Context,
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
       // docs:start:imports
        history::{
            contract_inclusion::{
                prove_contract_inclusion,
            },  
            note_inclusion::{
                prove_note_commitment_inclusion,
                prove_note_inclusion,
            },
            note_validity::{
                prove_note_validity,
            },
            nullifier_inclusion::{
                prove_nullifier_inclusion,
            },
            nullifier_non_inclusion::{
                prove_nullifier_non_inclusion,
                prove_note_not_nullified,
            },
            public_value_inclusion::{
                prove_public_value_inclusion,
            },
        },
         // docs:end:imports
    };
    // docs:start:value_note_imports
    use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};
    // docs:end:value_note_imports
    struct Storage {
        private_values: Map<AztecAddress, Set<ValueNote, VALUE_NOTE_LEN>>,
        public_value: PublicState<Field, 1>,
    }

    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                private_values: Map::new(
                    context,
                    1, // Storage slot
                    |context, slot| {
                        Set::new(context, slot, ValueNoteMethods)
                    },
                ),
                public_value: PublicState::new(
                    context,
                    2, // Storage slot
                    FieldSerializationMethods,
                ),
            }
        }
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

    // Proves that the owner owned a ValueNote at block `block_number`.
    #[aztec(private)]
    fn test_note_inclusion_proof(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the note exists
        // Value bellow is only used when the note is not found --> used to test the note inclusion failure case (it
        // allows me to pass in random value of note nullifier - I cannot add and fetch a random note from PXE because
        // PXE performs note commitment inclusion check when you add a new note).
        spare_commitment: Field
    ) {
        // docs:start:get_note_from_pxe
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner);
        let options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];
        // docs:end:get_note_from_pxe

        // 2) Prove the note inclusion
        if maybe_note.is_some() {
            // docs:start:prove_note_inclusion
            prove_note_inclusion(
                ValueNoteMethods,
                maybe_note.unwrap_unchecked(),
                block_number,
                context
            );
            // docs:end:prove_note_inclusion
        } else {
            // Note was not found so we will prove inclusion of the spare commitment
            prove_note_commitment_inclusion(spare_commitment, block_number, context);
        };
    }

    // Proves that the note was not yet nullified at block `block_number`.
    #[aztec(private)]
    fn test_nullifier_non_inclusion_proof(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the nullifier does not exists
        // Value bellow is only used when the note is not found --> used to test the nullifier non-inclusion failure
        // case (it allows me to pass in random value of note nullifier - I cannot add and fetch a random note from PXE
        // because PXE performs note commitment inclusion check when you add a new note).
        spare_nullifier: Field
    ) {
        // 2) Get the note from PXE
        let private_values = storage.private_values.at(owner);
        let options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 3) Compute the nullifier from the note
        if maybe_note.is_some() {
            // docs:start:prove_note_not_nullified
            prove_note_not_nullified(
                ValueNoteMethods,
                maybe_note.unwrap_unchecked(),
                block_number,
                &mut context
            );
            // docs:end:prove_note_not_nullified
        } else {
            // Note was not found so we will use the spare nullifier
            // docs:start:prove_nullifier_non_inclusion
            prove_nullifier_non_inclusion(spare_nullifier, block_number, context);
            // docs:end:prove_nullifier_non_inclusion
        };
    }

    #[aztec(private)]
    fn test_note_validity_proof(
        owner: AztecAddress,
        block_number: u32 // The block at which we'll prove that the note exists and is not nullified
    ) {
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner);
        let options = NoteGetterOptions::new().select(1, owner.to_field(), Option::none()).set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

        // 2) Prove the note validity
        // docs:start:prove_note_validity
        prove_note_validity(ValueNoteMethods, note, block_number, &mut context);
        // docs:end:prove_note_validity
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
    fn test_nullifier_inclusion_proof(
        nullifier: Field,
        block_number: u32 // The block at which we'll prove that the nullifier not exists in the tree
    ) {
        // docs:start:prove_nullifier_inclusion
        prove_nullifier_inclusion(nullifier, block_number, context);
        // docs:end:prove_nullifier_inclusion
    }

    #[aztec(private)]
    fn test_public_value_inclusion_proof(
        public_value: Field,
        block_number: u32 // The block at which we'll prove that the public value exists
    ) {
        prove_public_value_inclusion(
            public_value,
            storage.public_value.storage_slot,
            block_number,
            context
        );
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
    fn test_contract_inclusion_proof(
        deployer_public_key: GrumpkinPoint,
        contract_address_salt: Field,
        function_tree_root: Field,
        constructor_hash: Field,
        portal_contract_address: EthAddress,
        block_number: u32 // The block at which we'll prove that the public value exists
    ) {
        let proven_contract_address = prove_contract_inclusion(
            deployer_public_key,
            contract_address_salt,
            function_tree_root,
            constructor_hash,
            portal_contract_address,
            block_number,
            context
        );
        // Here typically the factory would add the contract address to its internal map of deployed contracts.
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, serialized_note)
    }
}
