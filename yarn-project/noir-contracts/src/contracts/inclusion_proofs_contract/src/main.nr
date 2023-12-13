// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::protocol_types::address::AztecAddress;
    use dep::aztec::{
        state_vars::{
            map::Map,
            set::Set,
            public_state::PublicState,
        },
        selector::compute_selector,
        types::{
            type_serialization::field_serialization::FieldSerializationMethods,
        },
        context::Context,
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        
        history::{
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
    };
    use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};

    struct Storage {
        private_values: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
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
        let selector = compute_selector("_initialize(Field)");
        context.call_public_function(context.this_address(), selector, [public_value]);
    }

    #[aztec(public)]
    internal fn _initialize(value: Field) {
        storage.public_value.write(value);
    }

    // Creates a value note owned by `owner`.
    #[aztec(private)]
    fn create_note(
        owner: AztecAddress,
        value: Field,
    ) {
        let owner_private_values = storage.private_values.at(owner.to_field());
        let mut note = ValueNote::new(value, owner);
        owner_private_values.insert(&mut note, true);
    }


    // Proves that the owner owned a ValueNote at block `block_number`.
    #[aztec(private)]
    fn test_note_inclusion_proof(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the note exists
        spare_commitment: Field, // This is only used when the note is not found --> used to test the failure case
    ) {
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner.to_field());
        let options = NoteGetterOptions::new().select(1, owner.to_field()).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 2) Prove the note inclusion
        if maybe_note.is_some() {
            prove_note_inclusion(ValueNoteMethods, maybe_note.unwrap_unchecked(), block_number, context);
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
        spare_nullifier: Field, // This is only used when the note is not found --> used to test the failure case
    ) {
        // 2) Get the note from PXE
        let private_values = storage.private_values.at(owner.to_field());
        let options = NoteGetterOptions::new().select(1, owner.to_field()).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 3) Compute the nullifier from the note
        if maybe_note.is_some() {
            prove_note_not_nullified(ValueNoteMethods, maybe_note.unwrap_unchecked(), block_number, context);
        } else {
            // Note was not found so we will use the spare nullifier
            prove_nullifier_non_inclusion(spare_nullifier, block_number, context);
        };
    }

    #[aztec(private)]
    fn test_note_validity_proof(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the note exists and is not nullified
    ) {
        // 1) Get the note from PXE.
        let private_values = storage.private_values.at(owner.to_field());
        let options = NoteGetterOptions::new().select(1, owner.to_field()).set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

        // 2) Prove the note validity
        prove_note_validity(ValueNoteMethods, note, block_number, context);
    }

    #[aztec(private)]
    fn nullify_note(
        owner: AztecAddress,
    ) {
        let private_values = storage.private_values.at(owner.to_field());
        let options = NoteGetterOptions::new().select(1, owner.to_field()).set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

        private_values.remove(note);
    }

    // Proves nullifier existed at block `block_number`.
    // Note: I am not getting a nullifier of the note that was created in this contract in this function because it is
    //       currently not possible to obtain a nullified note from PXE.
    #[aztec(private)]
    fn test_nullifier_inclusion_proof(
        nullifier: Field,
        block_number: u32, // The block at which we'll prove that the nullifier not exists in the tree
    ) {
        prove_nullifier_inclusion(nullifier, block_number, context);
    }

    #[aztec(private)]
    fn test_public_value_inclusion_proof(
        public_value: Field,
        block_number: u32, // The block at which we'll prove that the public value exists
    ) {
        prove_public_value_inclusion(public_value, storage.public_value.storage_slot, block_number, context);
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, serialized_note: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let _address = AztecAddress::from_field(contract_address); // TODO(benesjan) https://github.com/AztecProtocol/aztec-packages/issues/3669
        let note_header = NoteHeader::new(_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, serialized_note)
    }
}