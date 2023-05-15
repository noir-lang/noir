contract Parent {
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::CallContext;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::public_call;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec3::public_call_stack_item::PublicCallStackItem;
    
    fn constructor(
        inputs: pub Inputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }

    fn entryPoint(
        inputs: pub Inputs,
        targetContract: pub Field,
        targetSelector: pub Field,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut initialContext = PrivateFunctionContext::new();
        initialContext.args = initialContext.args.push_array([targetContract, targetSelector]);

        let (callStackItem, mut context) = PrivateCallStackItem::call(targetContract, targetSelector, [0; dep::aztec3::abi::MAX_ARGS], initialContext);

        let result = callStackItem.public_inputs.return_values[0];

        context.return_values = context.return_values.push(result);

        context.finish(inputs)
    }

    open fn pubEntryPoint(_call_context: CallContext, targetContract: Field, targetSelector: Field, initValue: Field, _padding: [Field; dep::aztec3::abi::MAX_ARGS - 3]) -> pub Field {
        // The call_public_function util always receives and returns MAX_ARGS and MAX_RETURN_VALUES. The caller
        // needs to pad the arguments and extract the return values it needs. The callee needs to always accept
        // MAX_ARGS, since the oracle doesn't have the info to know how many args to pass. However, the callee
        // can return however many args it wants, since the oracle can take care of the padding.
        let mut args = [0; dep::aztec3::abi::MAX_ARGS];
        args[0] = initValue;
        public_call::call_public_function(targetContract, targetSelector, args)[0]
    }

    fn enqueueCallToChild(
        inputs: pub Inputs,
        targetContract: pub Field,
        targetSelector: pub Field,
        targetValue: pub Field,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut initialContext = PrivateFunctionContext::new();
        initialContext.args = initialContext.args.push_array([targetContract, targetSelector, targetValue]);

        let mut args = [0; dep::aztec3::abi::MAX_ARGS];
        args[0] = targetValue;
        let (_callStackItem, mut context) = PublicCallStackItem::call(targetContract, targetSelector, args, initialContext);

        context.finish(inputs)
    }
}