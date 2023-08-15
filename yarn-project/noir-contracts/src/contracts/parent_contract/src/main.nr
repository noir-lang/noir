// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::{
        PrivateContext,
        PublicContext,
    };

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateContext::new(inputs, 0).finish()
    }

    // Private function to call another private function in the targetContract using the provided selector
    fn entryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
        ]));

        // Call the target private function
        let return_values = context.call_private_function(targetContract, targetSelector, [0]);

        // Copy the return value from the call to this function's return values
        let result = return_values[0];
        context.return_values.push(result);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Public function to directly call another public function to the targetContract using the selector and value provided
    open fn pubEntryPoint(
        inputs: PublicContextInputs,
        targetContract: Field,
        targetSelector: Field,
        initValue: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([targetContract, targetSelector, initValue]));

        let return_values = context.call_public_function(targetContract, targetSelector, [initValue]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

    // Same as pubEntryPoint, but calls the target contract twice, using the return value from the first invocation as the argument for the second.
    open fn pubEntryPointTwice(
        inputs: PublicContextInputs,
        targetContract: Field,
        targetSelector: Field,
        initValue: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([targetContract, targetSelector, initValue]));

        let returnValue = context.call_public_function(targetContract, targetSelector, [initValue])[0];
        let return_values = context.call_public_function(targetContract, targetSelector, [returnValue]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

    // Private function to enqueue a call to the targetContract address using the selector and argument provided
    fn enqueueCallToChild(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        context.call_public_function(targetContract, targetSelector, [targetValue]);

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
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        // Enqueue the first public call
        context.call_public_function(targetContract, targetSelector, [targetValue]);
        // Enqueue the second public call
        context.call_public_function(targetContract, targetSelector, [targetValue]);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Private function to enqueue a call to the pubEntryPoint function of this same contract, passing the target arguments provided
    fn enqueueCallToPubEntryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = context.this_address();
        let _void = context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Private function to enqueue two calls to the pubEntryPoint function of this same contract, passing the target arguments provided
    fn enqueueCallsToPubEntryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = context.this_address();
        
        context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);

        context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue + 1]);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }
}
