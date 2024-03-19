// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, PublicMutable, PrivateSet, PrivateContext, emit_unencrypted_log};

    use dep::aztec::{
        context::{PublicContext, Context}, protocol_types::{abis::{call_context::CallContext}},
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader}
    };
    use dep::value_note::value_note::ValueNote;

    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    #[aztec(private)]
    fn value(input: Field) -> Field {
        input + context.chain_id() + context.version()
    }
    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values. 
    // Can only be called from this contract.
    #[aztec(private)]
    #[aztec(internal)]
    fn valueInternal(input: Field) -> Field {
        input + context.chain_id() + context.version()
    }

    // Returns base_value + chain_id + version + block_number + timestamp
    #[aztec(public)]
    fn pubGetValue(base_value: Field) -> Field {
        let returnValue = base_value
            + context.chain_id()
            + context.version()
            + context.block_number()
            + context.timestamp();

        returnValue
    }

    // Sets `current_value` to `new_value`
    #[aztec(public)]
    fn pubSetValue(new_value: Field) -> Field {
        storage.current_value.write(new_value);
        emit_unencrypted_log(&mut context, new_value);

        new_value
    }

    #[aztec(private)]
    fn privateSetValue(new_value: Field, owner: AztecAddress) -> Field {
        let mut note = ValueNote::new(new_value, owner);
        storage.a_private_value.insert(&mut note, true);
        new_value
    }

    #[aztec(private)]
    fn privateGetValue(amount: Field, owner: AztecAddress) -> Field {
        let options = NoteGetterOptions::new().select(ValueNote::properties().value, amount, Option::none()).select(
            ValueNote::properties().owner,
            owner.to_field(),
            Option::none()
        ).set_limit(1);
        let notes = storage.a_private_value.get_notes(options);
        notes[0].unwrap_unchecked().value
    }

    // Increments `current_value` by `new_value`
    #[aztec(public)]
    fn pubIncValue(new_value: Field) -> Field {
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        emit_unencrypted_log(&mut context, new_value);

        new_value
    }

    // Increments `current_value` by `new_value`. Can only be called from this contract.
    #[aztec(public)] 
    #[aztec(internal)]
    fn pubIncValueInternal(new_value: Field) -> Field {
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        emit_unencrypted_log(&mut context, new_value);

        new_value
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedFirst() {
        let pubSetValueSelector = FunctionSelector::from_signature("pubSetValue(Field)");
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);

        storage.current_value.write(20);
        emit_unencrypted_log(&mut context, 20);
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedLast() {
        storage.current_value.write(20);
        emit_unencrypted_log(&mut context, 20);

        let pubSetValueSelector = FunctionSelector::from_signature("pubSetValue(Field)");
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);
    }
}
