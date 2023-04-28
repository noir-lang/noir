contract Child {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::context::PrivateFunctionContext;

    /// ABI constructor type "secret"
    /// ABI constructor params []
    /// ABI constructor return []
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }

    /// ABI value type "secret"
    /// ABI value params [{"name":"input","type":{"kind":"field"},"visibility":"public"}]
    /// ABI value return [{"kind":"field"}]
    fn value(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut context = PrivateFunctionContext::new();

        context.return_values = context.return_values.push(inputs.args[0] + 42);

        context.finish(inputs)
    }
}
