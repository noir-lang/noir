// A contract used for testing a random hodgepodge of small features from simulator and end-to-end tests.
contract StatefulTest {
    use dep::aztec::prelude::{PrivateContext, NoteHeader, Map, PublicMutable, PrivateSet, AztecAddress, FunctionSelector};
    use dep::value_note::{balance_utils, utils::{increment, decrement}, value_note::{VALUE_NOTE_LEN, ValueNote}};
    use dep::aztec::{
        deploy::deploy_contract as aztec_deploy_contract, context::{PublicContext, Context},
        oracle::get_contract_instance::get_contract_instance, initializer::assert_is_initialized_private
    };

    struct Storage {
        notes: Map<AztecAddress, PrivateSet<ValueNote>>,
        public_values: Map<AztecAddress, PublicMutable<Field>>,
    }

    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(owner: AztecAddress, value: Field) {
        let selector = FunctionSelector::from_signature("create_note_no_init_check((Field),Field)");
        let _res = context.call_private_function(context.this_address(), selector, [owner.to_field(), value]);
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn public_constructor(owner: AztecAddress, value: Field) {
        let selector = FunctionSelector::from_signature("increment_public_value_no_init_check((Field),Field)");
        let _res = context.call_public_function(context.this_address(), selector, [owner.to_field(), value]);
    }

    #[aztec(private)]
    fn create_note(owner: AztecAddress, value: Field) {
        if (value != 0) {
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    #[aztec(noinitcheck)]
    fn create_note_no_init_check(owner: AztecAddress, value: Field) {
        if (value != 0) {
            let loc = storage.notes.at(owner);
            increment(loc, value, owner);
        }
    }

    #[aztec(private)]
    fn destroy_and_create(recipient: AztecAddress, amount: Field) {
        assert_is_initialized_private(&mut context);
        let sender = context.msg_sender();

        let sender_notes = storage.notes.at(sender);
        decrement(sender_notes, amount, sender);

        let recipient_notes = storage.notes.at(recipient);
        increment(recipient_notes, amount, recipient);
    }

    #[aztec(private)]
    #[aztec(noinitcheck)]
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

    #[aztec(public)]
    #[aztec(noinitcheck)]
    fn increment_public_value_no_init_check(owner: AztecAddress, value: Field) {
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
