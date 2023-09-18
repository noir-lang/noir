// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract StatefulTest {
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        state_vars::{map::Map, public_state::PublicState, set::Set},
        types::type_serialisation::field_serialisation::{
            FieldSerialisationMethods, FIELD_SERIALISED_LEN,
        },
    };

    struct Storage {
        notes: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
        public_values: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                notes: Map::new(
                    context,
                    1, // Storage slot
                    |context, slot| {
                        Set::new(context, slot, ValueNoteMethods)
                    },
                ),
                public_values: Map::new(
                    context,
                    2,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerialisationMethods,
                        )
                    },
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor(owner: Field, value: Field) {
        let storage = Storage::init(Context::private(&mut context));
        let loc = storage.notes.at(owner);
        increment(loc, value, owner);
    }

    #[aztec(private)]
    fn create_note(
        owner: Field,
        value: Field, 
    )  {
        let storage = Storage::init(Context::private(&mut context));
        if (value != 0){
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    fn destroy_and_create(
        recipient: Field,
        amount: Field, 
    )  {
        let storage = Storage::init(Context::private(&mut context));
        let sender = context.msg_sender();

        let sender_notes = storage.notes.at(sender);
        decrement(sender_notes, amount, sender);

        let recipient_notes = storage.notes.at(recipient);
        increment(recipient_notes, amount, recipient);
    }

    unconstrained fn summed_values(
        owner: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());
        let owner_balance = storage.notes.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance)
    }

    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
    }
}
