contract Parent {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;

    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        Outputs::new().finish(inputs)
    }

    fn entryPoint(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let targetContract = inputs.args[0];
        let targetSelector = inputs.args[1];

        let mut initialOutputs = Outputs::new();

        let (callStackItem, mut outputs) = PrivateCallStackItem::call(targetContract, targetSelector, [0; dep::aztec3::abi::MAX_ARGS], initialOutputs);

        let result = callStackItem.public_inputs.return_values[0];

        outputs.return_values = outputs.return_values.push(result);

        outputs.finish(inputs)
    }
}

contract Child {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;

    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        Outputs::new().finish(inputs)
    }

    fn value(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut outputs = Outputs::new();

        outputs.return_values = outputs.return_values.push(inputs.args[0] + 42);

        outputs.finish(inputs)
    }
}
