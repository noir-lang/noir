mod storage;

// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use crate::storage::Storage;
    use dep::aztec::oracle::logs::emit_unencrypted_log;

    #[aztec(private)]
    fn constructor()  {}

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    #[aztec(private)]
    fn value(
        input: Field,
    ) {
        context.return_values.push(input + context.chain_id() + context.version());
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values. 
    // Can only be called from this contract.
    #[aztec(private)]
    fn valueInternal(
        input: Field,
    ) {
        assert(inputs.call_context.msg_sender == inputs.call_context.storage_contract_address);
        context.return_values.push(input + context.chain_id() + context.version());
    }

    // Returns base_value + 42.
    #[aztec(public)]
    fn pubGetValue(base_value: Field) {
        let returnValue = base_value + context.chain_id() + context.version() + context.block_number() + context.timestamp();

        context.return_values.push(returnValue);
    }

    // Sets `current_value` to `new_value`
    #[aztec(public)]
    fn pubSetValue(new_value: Field) {
        let storage = Storage::init();
        storage.current_value.write(new_value);
        let _hash = emit_unencrypted_log(new_value);
        context.return_values.push(new_value);
    }

    // Increments `current_value` by `new_value`
    #[aztec(public)]
    fn pubIncValue(new_value: Field) {
        let storage = Storage::init();
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        let _hash = emit_unencrypted_log(new_value);
        context.return_values.push(new_value);
    }

    // Increments `current_value` by `new_value`. Can only be called from this contract.
    #[aztec(public)]
    fn pubIncValueInternal(new_value: Field) {
        let storage = Storage::init();
        assert(inputs.call_context.msg_sender == inputs.call_context.storage_contract_address);
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        let _hash = emit_unencrypted_log(new_value);
        context.return_values.push(new_value);
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedFirst() {
        let pubSetValueSelector = 0x5b0f91b0;
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);

        let storage = Storage::init();
        storage.current_value.write(20);
        let _hash = emit_unencrypted_log(20);
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedLast() {
        let storage = Storage::init();
        storage.current_value.write(20);
        let _hash = emit_unencrypted_log(20);

        let pubSetValueSelector = 0x5b0f91b0;
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);
    }
}
