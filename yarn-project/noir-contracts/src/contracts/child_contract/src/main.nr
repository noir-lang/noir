contract Child {
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::abi::PublicContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::state_vars::storage_field::StorageField;

    global current_value = StorageField{ storage_slot: 1 };

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateFunctionContext::new().finish(inputs)
    }

    fn value(
        inputs: PrivateContextInputs,
        input: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push(input);

        context.return_values = context.return_values.push(input + inputs.private_global_variables.chain_id + inputs.private_global_variables.version);

        context.finish(inputs)
    }

    // Nested public functions always get called with MAX_ARGS, since we don't have the ABI available
    // during execution time to know how many args are expected, hence the _padding argument. We should
    // be able to remove it when we migrate to brillig.
    open fn pubValue(_inputs: PublicContextInputs, base_value: Field, _padding: [Field; abi::MAX_ARGS - 1]) -> pub Field {
        base_value + 42
    }

    // Same as above, we need the padding argument for open functions.
    open fn pubStoreValue(_inputs: PublicContextInputs, new_value: Field, _padding: [Field; abi::MAX_ARGS - 1]) -> pub Field {
        // Compiler complains if we don't assign the result to anything
        let old_value = current_value.read();
        let _void1 = current_value.write(old_value + new_value);
        // Compiler fails with "we do not allow private ABI inputs to be returned as public outputs" if we try to
        // return new_value as-is, but then it also complains if we add `pub` to `new_value` in the args, so we
        // just assign it to another variable and tweak it so it's not the same value, and the compiler is happy.
        let ret_value = new_value + 1;
        ret_value
    }
}
