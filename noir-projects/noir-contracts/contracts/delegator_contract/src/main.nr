// A contract used along with `Parent` contract to test nested calls.
contract Delegator {
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteViewerOptions, emit_unencrypted_log,
        PublicMutable, PrivateSet
    };
    use dep::value_note::value_note::ValueNote;

    #[aztec(storage)]
    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    #[aztec(private)]
    fn private_delegate_set_value(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 2]
    ) {
        // Call the target private function
        let return_values = context.delegate_call_private_function(target_contract, target_selector, args);
        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    #[aztec(private)]
    fn enqueued_delegate_set_value(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) {
        context.delegate_call_public_function(target_contract, target_selector, args);
    }

    #[aztec(public)]
    fn public_delegate_set_value(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) {
        let _ = context.delegate_call_public_function(target_contract, target_selector, args);
    }

    unconstrained fn view_private_value(amount: Field, owner: AztecAddress) -> pub Field {
        let mut options = NoteViewerOptions::new();
        options = options.select(ValueNote::properties().value, amount, Option::none()).select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        let notes = storage.a_private_value.view_notes(options);
        notes[0].unwrap_unchecked().value
    }

    unconstrained fn view_public_value() -> pub Field {
        storage.current_value.read()
    }
}
