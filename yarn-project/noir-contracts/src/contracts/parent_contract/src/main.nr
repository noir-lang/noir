// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::Context;
    use dep::aztec::public_call_stack_item::call_public_function;

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        Context::new(inputs, 0).finish()
    }

    // Private function to call another private function in the targetContract using the provided selector
    fn entryPoint(
        inputs: PrivateContextInputs,
        targetContract: Field,
        targetSelector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([
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
        let mut context = Context::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let _callStackItem = context.call_public_function(targetContract, targetSelector, [targetValue]);

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
        let mut context = Context::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        // Enqueue the first public call
        let return_values1 = context.call_public_function(targetContract, targetSelector, [targetValue]);
        // Enqueue the second public call
        let _return_values2 = context.call_public_function(targetContract, targetSelector, [return_values1[0]]);

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
        let mut context = Context::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = inputs.call_context.storage_contract_address;
        let _return_values = context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);

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
        let mut context = Context::new(inputs, abi::hash_args([
            targetContract,
            targetSelector,
            targetValue,
        ]));

        let pubEntryPointSelector = 3221316504;
        let thisAddress = inputs.call_context.storage_contract_address;
        
        let _return_values1 = context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);

        let _return_values2 = context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue + 1]);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }
}
