contract EmptyConstructor {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;
    use dep::aztec3::abi::FinalOutputs;

    fn constructor(
        inputs: pub Inputs,
    ) -> pub FinalOutputs {
        for i in 0..abi::MAX_ARGS {
            constrain inputs.args[i] == 0;
        }

        let mut outputs = Outputs::new();

        outputs.finish(inputs)
    }
}
