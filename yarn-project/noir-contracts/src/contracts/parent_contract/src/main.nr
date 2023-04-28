contract Parent {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;

    /// ABI constructor type "secret"
    /// ABI constructor params []
    /// ABI constructor return []
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }

    /// ABI entryPoint type "secret"
    /// ABI entryPoint params [{"name":"targetContract","type":{"kind":"field"},"visibility":"public"},{"name":"targetSelector","type":{"kind":"field"}}]
    /// ABI entryPoint return [{"kind":"field"}]
    fn entryPoint(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let targetContract = inputs.args[0];
        let targetSelector = inputs.args[1];

        let mut initialContext = PrivateFunctionContext::new();

        let (callStackItem, mut context) = PrivateCallStackItem::call(targetContract, targetSelector, [0; dep::aztec3::abi::MAX_ARGS], initialContext);

        let result = callStackItem.public_inputs.return_values[0];

        context.return_values = context.return_values.push(result);

        context.finish(inputs)
    }
}