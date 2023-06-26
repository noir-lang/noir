contract Test {
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;

    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> distinct pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }
}
