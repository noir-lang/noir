// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::abi::PublicContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::oracle::public_call;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec3::public_call_stack_item::PublicCallStackItem;
    use dep::aztec3::public_call_stack_item::call_public_function;

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateFunctionContext::new(inputs, 0).finish()
    }

    // Private function to call another private function in the targetContract using the provided selector
    fn entryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
        ]));

        // Call the target private function
        let (callStackItem, mut context) = PrivateCallStackItem::call(targetContract, targetSelector, [0], initialContext);

        // Copy the return value from the call to this function's return values
        let result = callStackItem.public_inputs.return_values[0];
        context.return_values = context.return_values.push(result);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Public function to directly call another public function to the targetContract using the selector and value provided
    open fn pubEntryPoint(_inputs: PublicContextInputs, targetContract: Field, targetSelector: Field, initValue: Field) -> pub Field {
        call_public_function(targetContract, targetSelector, [initValue])[0]
    }

    // Same as pubEntryPoint, but calls the target contract twice, using the return value from the first invocation as the argument for the second.
    open fn pubEntryPointTwice(_inputs: PublicContextInputs, targetContract: Field, targetSelector: Field, initValue: Field) -> pub Field {
        let returnValue = call_public_function(targetContract, targetSelector, [initValue])[0];
        call_public_function(targetContract, targetSelector, [returnValue])[0]
    }

    // Private function to enqueue a call to the targetContract address using the selector and argument provided
    fn enqueueCallToChild(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let (_callStackItem, mut context) = PublicCallStackItem::call(targetContract, targetSelector, [targetValue], initialContext);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Private function to enqueue a call to the targetContract address using the selector and argument provided
    fn enqueueCallToChildTwice(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context0 = PrivateFunctionContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        // Enqueue the first public call
        let (callStackItem1, context1) = PublicCallStackItem::call(targetContract, targetSelector, [targetValue], context0);
        // Enqueue the second public call
        let (_callStackItem2, context2) = PublicCallStackItem::call(targetContract, targetSelector, [callStackItem1.public_inputs.return_values[0]], context1);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context2.finish()
    }

    // Private function to enqueue a call to the pubEntryPoint function of this same contract, passing the target arguments provided
    fn enqueueCallToPubEntryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = inputs.call_context.storage_contract_address;
        let (_callStackItem1, mut context1) = PublicCallStackItem::call(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue], initialContext);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context1.finish()
    }

    // Private function to enqueue two calls to the pubEntryPoint function of this same contract, passing the target arguments provided
    fn enqueueCallsToPubEntryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = inputs.call_context.storage_contract_address;
        
        let (_callStackItem1, mut context1) = PublicCallStackItem::call(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue], initialContext);

        let (_callStackItem2, mut context2) = PublicCallStackItem::call(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue + 1], context1);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context2.finish()
    }
}