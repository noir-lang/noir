contract Child {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;

    /// ABI constructor params []
    /// ABI constructor return []
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        Outputs::new().finish(inputs)
    }

    /// ABI value params [{"name":"input","type":{"kind":"field"},"visibility":"public"}]
    /// ABI value return [{"kind":"field"}]
    fn value(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut outputs = Outputs::new();

        outputs.return_values = outputs.return_values.push(inputs.args[0] + 42);

        outputs.finish(inputs)
    }
}
