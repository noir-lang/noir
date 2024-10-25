// Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
// Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
// Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
// Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
contract Benchmarking {
    use aztec::protocol_types::abis::function_selector::FunctionSelector;

    use value_note::{
        utils::{decrement, increment},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };

    use aztec::{
        context::Context,
        log::emit_unencrypted_log,
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader},
        state_vars::{Map, PrivateSet, PublicMutable},
        types::{
            address::AztecAddress,
            type_serialization::field_serialization::{
                FIELD_SERIALIZED_LEN, FieldSerializationMethods,
            },
        },
    };

    struct Storage {
        notes: Map<AztecAddress, PrivateSet<ValueNote, VALUE_NOTE_LEN>>,
        balances: Map<AztecAddress, PublicMutable<Field, FIELD_SERIALIZED_LEN>>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                notes: Map::new(
                    context,
                    1,
                    |context, slot| PrivateSet::new(context, slot, ValueNoteMethods),
                ),
                balances: Map::new(
                    context,
                    2,
                    |context, slot| {
                        PublicMutable::new(context, slot, FieldSerializationMethods)
                    },
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor() {}

    // Nec tincidunt praesent semper feugiat nibh sed pulvinar. Nibh nisl condimentum id venenatis a.
    #[aztec(private)]
    fn create_note(owner: Field, value: Field) {
        increment(storage.notes.at(owner), value, owner);
    }

    // Diam quam nulla porttitor massa id. Elit ullamcorper dignissim cras tincidunt lobortis feugiat.
    #[aztec(private)]
    fn recreate_note(owner: Field, index: u32) {
        let owner_notes = storage.notes.at(owner);
        let getter_options = NoteGetterOptions::new().set_limit(1).set_offset(index);
        let notes = owner_notes.get_notes(getter_options);
        let note = notes[0].unwrap_unchecked();
        owner_notes.remove(note);
        increment(owner_notes, note.value, owner);
    }

    // Ultrices in iaculis nunc sed augue lacus.
    #[aztec(public)]
    fn increment_balance(owner: Field, value: Field) {
        let current = storage.balances.at(owner).read();
        storage.balances.at(owner).write(current + value);
        let _callStackItem1 = context.call_public_function(
            context.this_address(),
            comptime { FunctionSelector::from_signature("broadcast(Field)") },
            [owner],
        );
    }

    // Est ultricies integer quis auctor elit sed. In nibh mauris cursus mattis molestie a iaculis.
    #[aztec(public)]
    fn broadcast(owner: Field) {
        emit_unencrypted_log(&mut context, storage.balances.at(owner).read());
    }
}

// Uses the token bridge contract, which tells which input token we need to talk to and handles the exit funds to L1
contract Uniswap {}
