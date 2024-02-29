// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract StatefulTest {
    use dep::std::option::Option;
    use dep::value_note::{balance_utils, utils::{increment, decrement}, value_note::{VALUE_NOTE_LEN, ValueNote}};
    use dep::aztec::{
        deploy::{deploy_contract as aztec_deploy_contract},
        context::{PrivateContext, PublicContext, Context},
        note::{note_header::NoteHeader, utils as note_utils},
        state_vars::{Map, PublicMutable, PrivateSet},
        oracle::get_contract_instance::get_contract_instance,
        initializer::{mark_as_initialized, assert_is_initialized},
        protocol_types::{address::AztecAddress, abis::function_selector::FunctionSelector},
    };

    struct Storage {
        notes: Map<AztecAddress, PrivateSet<ValueNote>>,
        public_values: Map<AztecAddress, PublicMutable<Field>>,
    }

    #[aztec(private)]
    fn constructor(owner: AztecAddress, value: Field) {
        let selector = FunctionSelector::from_signature("create_note_no_init_check((Field),Field)");
        let _res = context.call_private_function(context.this_address(), selector, [owner.to_field(), value]);
        mark_as_initialized(&mut context);
    }

    #[aztec(private)]
    fn create_note(owner: AztecAddress, value: Field) {
        assert_is_initialized(&mut context);
        if (value != 0) {
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    fn create_note_no_init_check(owner: AztecAddress, value: Field) {
        if (value != 0) {
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    fn destroy_and_create(recipient: AztecAddress, amount: Field) {
        assert_is_initialized(&mut context);
        let sender = context.msg_sender();

        let sender_notes = storage.notes.at(sender);
        decrement(sender_notes, amount, sender);

        let recipient_notes = storage.notes.at(recipient);
        increment(recipient_notes, amount, recipient);
    }

    #[aztec(private)]
    fn destroy_and_create_no_init_check(recipient: AztecAddress, amount: Field) {
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
}