contract Test {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::context::PrivateFunctionContext;

    /// ABI constructor type "secret"
    /// ABI constructor params []
    /// ABI constructor return []
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        for i in 0..abi::MAX_ARGS {
            constrain inputs.args[i] == 0;
        }

        let mut context = PrivateFunctionContext::new();

        context.finish(inputs)
    }
}
