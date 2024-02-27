// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use dep::std::option::Option;

    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context}, log::emit_unencrypted_log,
        state_vars::{PublicMutable, PrivateSet},
        protocol_types::{abis::{function_selector::FunctionSelector, call_context::CallContext}, address::AztecAddress},
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader, utils as note_utils}
    };
    use dep::value_note::value_note::{ValueNote, VALUE_NOTE_LEN};

    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    #[aztec(private)]
    fn constructor() {}

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    #[aztec(private)]
    fn value(input: Field) -> Field {
        input + context.chain_id() + context.version()
    }

    fn check_sender(call_context: CallContext) {
        assert(
            call_context.msg_sender.eq(call_context.storage_contract_address), "Sender must be this contract"
        );
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values. 
    // Can only be called from this contract.
    #[aztec(private)]
    fn valueInternal(input: Field) -> Field {
        check_sender(inputs.call_context);
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
        let options = NoteGetterOptions::new().select(0, amount, Option::none()).select(1, owner.to_field(), Option::none()).set_limit(1);
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
    fn pubIncValueInternal(new_value: Field) -> Field {
        check_sender(inputs.call_context);
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
