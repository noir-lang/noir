contract Child {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::CallContext;
    use dep::aztec3::context::PrivateFunctionContext;
    
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }

    fn value(
        inputs: pub Inputs,
        input: pub Field,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push(input);

        context.return_values = context.return_values.push(input + 42);

        context.finish(inputs)
    }

    // Nested public functions always get called with MAX_ARGS, since we don't have the ABI available
    // during execution time to know how many args are expected, hence the _padding argument. We should
    // be able to remove it when we migrate to brillig.
    open fn pubValue(_call_context: CallContext, value: Field, _padding: [Field; dep::aztec3::abi::MAX_ARGS - 1]) -> pub Field {
        value + 42
    }
}
