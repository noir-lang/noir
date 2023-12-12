mod utils;

// A demonstration of inclusion and non-inclusion proofs.
contract InclusionProofs {
    use dep::protocol_types::constants::{
        NOTE_HASH_TREE_HEIGHT,
        PUBLIC_DATA_TREE_HEIGHT,
        GENERATOR_INDEX__PUBLIC_LEAF_INDEX,
    };
    use dep::std::merkle::compute_merkle_root;
    use dep::aztec::{
        state_vars::{
            map::Map,
            set::Set,
            public_state::PublicState,
        },
        selector::compute_selector,
        types::{
            address::AztecAddress,
            type_serialization::field_serialization::FieldSerializationMethods,
        },
        context::Context,
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        
        oracle::{
            get_membership_witness::{
                get_membership_witness,
                MembershipWitness,
            },
            get_sibling_path::get_sibling_path,
            get_nullifier_membership_witness::{
                get_low_nullifier_membership_witness,
                get_nullifier_membership_witness,
                NullifierMembershipWitness,
            },
        },
        hash::pedersen_hash,
    };
    use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};

    use crate::utils::{
        full_field_less_than,
        full_field_greater_than,
    };

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
        let owner_private_values = storage.private_values.at(owner.address);
        let mut note = ValueNote::new(value, owner.address);
        owner_private_values.insert(&mut note, true);
    }


    // Proves that the owner owned a ValueNote at block `block_number`.
    #[aztec(private)]
    fn proveNoteInclusion(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the note exists
        spare_commitment: Field, // This is only used when the note is not found --> used to test the failure case
    ) {
        // 1) Get block header from oracle and ensure that the block hash is included in the current blocks tree
        //    root.
        let block_header = context.get_block_header(block_number);

        // 2) Get the note from PXE.
        let private_values = storage.private_values.at(owner.address);
        let options = NoteGetterOptions::new().select(1, owner.address).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 3) Compute the commitment from the note
        let note_commitment = if maybe_note.is_some() {
            note_utils::compute_unique_siloed_note_hash(ValueNoteMethods, maybe_note.unwrap_unchecked())
        } else {
            // Note was not found so we will use the spare commitment
            spare_commitment
        };

        // 4) Get the membership witness of the note in the note hash tree
        let note_hash_tree_id = 2; // TODO(#3443)
        let witness: MembershipWitness<NOTE_HASH_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT + 1> =
            get_membership_witness(block_number, note_hash_tree_id, note_commitment);

        // 5) Prove that the commitment is in the note hash tree
        assert(
            block_header.note_hash_tree_root == compute_merkle_root(note_commitment, witness.index, witness.path),
            "Proving note inclusion failed"
        );

        // --> Now we have traversed the trees all the way up to blocks tree root.
    }

    // Proves that the note was not yet nullified at block `block_number`.
    #[aztec(private)]
    fn proveNullifierNonInclusion(
        owner: AztecAddress,
        block_number: u32, // The block at which we'll prove that the nullifier does not exists
        spare_nullifier: Field, // This is only used when the note is not found --> used to test the failure case
    ) {
        // 1) Get block header from oracle and ensure that the block hash is included in the current blocks tree
        //    root.
        let block_header = context.get_block_header(block_number);

        // 2) Get the note from PXE
        let private_values = storage.private_values.at(owner.address);
        let options = NoteGetterOptions::new().select(1, owner.address).set_limit(1);
        let notes = private_values.get_notes(options);
        let maybe_note = notes[0];

        // 3) Compute the nullifier from the note
        let nullifier = if maybe_note.is_some() {
            note_utils::compute_siloed_nullifier(ValueNoteMethods, maybe_note.unwrap_unchecked())
        } else {
            // Note was not found so we will use the spare nullifier
            spare_nullifier
        };

        // 4) Get the membership witness of a low nullifier of the nullifier
        let witness = get_low_nullifier_membership_witness(block_number, nullifier);

        // 5) Prove that the nullifier is not included in the nullifier tree

        // 5.a) Compute the low nullifier leaf and prove that it is in the nullifier tree
        let low_nullifier_leaf = witness.leaf_data.hash();
        assert(
            block_header.nullifier_tree_root == compute_merkle_root(low_nullifier_leaf, witness.index, witness.path),
            "Proving nullifier non-inclusion failed: Could not prove low nullifier inclusion"
        );

        // 5.b) Prove that the low nullifier is smaller than the nullifier
        assert(
            full_field_less_than(witness.leaf_data.value, nullifier),
            "Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed"
        );

        // 5.c) Prove that the low nullifier is pointing "over" the nullifier to prove that the nullifier is not
        // included in the nullifier tree (or to 0 if the to-be-inserted nullifier is the largest of all)
        assert(
            full_field_greater_than(witness.leaf_data.next_value, nullifier) | (witness.leaf_data.next_index == 0),
            "Proving nullifier non-inclusion failed: low_nullifier.next_value > nullifier.value check failed"
        );

        // --> Now we have traversed the trees all the way up to blocks tree root and verified that the nullifier
        //     was not yet included in the nullifier tree.
    }

    #[aztec(private)]
    fn nullifyNote(
        owner: AztecAddress,
    ) {
        let private_values = storage.private_values.at(owner.address);
        let options = NoteGetterOptions::new().select(1, owner.address).set_limit(1);
        let notes = private_values.get_notes(options);
        let note = notes[0].unwrap();

        private_values.remove(note);
    }

    // Proves nullifier existed at block `block_number`.
    // Note: I am not getting a nullifier of the note that was created in this contract in this function because it is
    //       currently not possible to obtain a nullified note from PXE.
    #[aztec(private)]
    fn proveNullifierInclusion(
        nullifier: Field,
        block_number: u32, // The block at which we'll prove that the nullifier not exists in the tree
    ) {
        // 1) Get block header from oracle and ensure that the block hash is included in the current blocks tree
        //    root.
        let block_header = context.get_block_header(block_number);

        // 2) Get the membership witness of the nullifier
        let witness = get_nullifier_membership_witness(block_number, nullifier);

        // 3) Check that the witness we obtained matches the nullifier
        assert(witness.leaf_data.value == nullifier, "Nullifier does not match value in witness");

        // 4) Compute the nullifier tree leaf
        let nullifier_leaf = witness.leaf_data.hash();

        // 5) Prove that the nullifier is in the nullifier tree
        assert(
            block_header.nullifier_tree_root == compute_merkle_root(nullifier_leaf, witness.index, witness.path),
            "Proving nullifier inclusion failed"
        );

        // --> Now we have traversed the trees all the way up to blocks tree root and verified that the nullifier
        //     was not yet included in the nullifier tree.
    }

    #[aztec(private)]
    fn provePublicValueInclusion(
        public_value: Field,
        block_number: u32, // The block at which we'll prove that the public value exists
    ) {
        // 1) Get block header from oracle and ensure that the block hash is included in the current blocks tree
        //    root.
        let block_header = context.get_block_header(block_number);

        // 2) Compute the public value leaf index.
        // We have to compute the leaf index here because unlike in the case of note commitments, public values are
        // not siloed with contract address so an oracle could cheat and give us a membership witness for arbitrary
        // value in the public data tree.
        let public_value_leaf_index = pedersen_hash(
            [context.this_address(), storage.public_value.storage_slot],
            GENERATOR_INDEX__PUBLIC_LEAF_INDEX
        );

        // 3) Get the sibling path of the public value leaf index in the public data tree at block `block_number`.
        let public_data_tree_id = 3; // TODO(#3443)
        let path: [Field; PUBLIC_DATA_TREE_HEIGHT] =
            get_sibling_path(block_number, public_data_tree_id, public_value_leaf_index);

        // 4) Prove that the public value provided on input is in the public data tree
        assert(
            block_header.public_data_tree_root == compute_merkle_root(public_value, public_value_leaf_index, path),
            "Proving public value inclusion failed"
        );

        // --> Now we have traversed the trees all the way up to blocks tree root and that way verified that
        //     a specific `public_value` was really set in a given contract storage slot at block `block_number`.
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, serialized_note: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, serialized_note)
    }
}