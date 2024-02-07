// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract StatefulTest {
    use dep::aztec::protocol_types::{
        address::AztecAddress,
        abis::function_selector::FunctionSelector,
    };
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNote},
    };
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        state_vars::{map::Map, public_state::PublicState, set::Set},
    };

    struct Storage {
        notes: Map<AztecAddress, Set<ValueNote>>,
        public_values: Map<AztecAddress, PublicState<Field>>,
    }

    #[aztec(private)]
    fn constructor(owner: AztecAddress, value: Field) {
        let selector = FunctionSelector::from_signature("create_note((Field),Field)");
        let _res = context.call_private_function(context.this_address(), selector, [owner.to_field(), value]);
    }

    #[aztec(private)]
    fn create_note(owner: AztecAddress, value: Field) {
        if (value != 0) {
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    fn destroy_and_create(recipient: AztecAddress, amount: Field) {
        let sender = context.msg_sender();

        let sender_notes = storage.notes.at(sender);
        decrement(sender_notes, amount, sender);

        let recipient_notes = storage.notes.at(recipient);
        increment(recipient_notes, amount, recipient);
    }

    #[aztec(public)]
    fn increment_public_value(owner: AztecAddress, value: Field) {
        let loc = storage.public_values.at(owner);
        loc.write(loc.read() + value);
    }

    unconstrained fn summed_values(owner: AztecAddress) -> pub Field {
        let owner_balance = storage.notes.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance)
    }

    unconstrained fn get_public_value(owner: AztecAddress) -> pub Field {
        storage.public_values.at(owner).read()
    }

    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
    }
}
