// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec::selector::compute_selector;

    #[aztec(private)]
    fn constructor() {}

    // Private function to call another private function in the targetContract using the provided selector
    #[aztec(private)]
    fn entryPoint(
        targetContract: Field,
        targetSelector: Field,
    ) -> Field {
        // Call the target private function
        let return_values = context.call_private_function(targetContract, targetSelector, [0]);

        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    // Public function to directly call another public function to the targetContract using the selector and value provided
    #[aztec(public)]
    fn pubEntryPoint(
        targetContract: Field,
        targetSelector: Field,
        initValue: Field
    ) -> Field {
        let return_values = context.call_public_function(targetContract, targetSelector, [initValue]);

        return_values[0]
    }

    // Same as pubEntryPoint, but calls the target contract twice, using the return value from the first invocation as the argument for the second.
    #[aztec(public)]
    fn pubEntryPointTwice(
        targetContract: Field,
        targetSelector: Field,
        initValue: Field
    ) -> Field {
        let returnValue = context.call_public_function(targetContract, targetSelector, [initValue])[0];
        let return_values = context.call_public_function(targetContract, targetSelector, [returnValue]);

        return_values[0]
    }

    // Private function to enqueue a call to the targetContract address using the selector and argument provided
    #[aztec(private)]
    fn enqueueCallToChild(
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) {
        context.call_public_function(targetContract, targetSelector, [targetValue]);
    }

    // Private function that enqueues two calls to a child contract: 
    // - one through a nested call to enqueueCallToChild with value 10,
    // - followed by one issued directly from this function with value 20.
    #[aztec(private)]
    fn enqueueCallsToChildWithNestedFirst(
        targetContract: Field,
        targetSelector: Field,
    ) {
        let enqueueCallToChildSelector = compute_selector("enqueueCallToChild(Field,Field,Field)");
        let _ret = context.call_private_function(context.this_address(), enqueueCallToChildSelector, [targetContract, targetSelector, 10]);
        context.call_public_function(targetContract, targetSelector, [20]);
    }

    // Private function that enqueues two calls to a child contract: 
    // - one issued directly from this function with value 20,
    // - followed by one through a nested call to enqueueCallToChild with value 10.
    #[aztec(private)]
    fn enqueueCallsToChildWithNestedLast(
        targetContract: Field,
        targetSelector: Field,
    ) {
        context.call_public_function(targetContract, targetSelector, [20]);
        let enqueueCallToChildSelector = compute_selector("enqueueCallToChild(Field,Field,Field)");
        let _ret = context.call_private_function(context.this_address(), enqueueCallToChildSelector, [targetContract, targetSelector, 10]);
    }

    // Private function to enqueue a call to the targetContract address using the selector and argument provided
    #[aztec(private)]
    fn enqueueCallToChildTwice(
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) {
        // Enqueue the first public call
        context.call_public_function(targetContract, targetSelector, [targetValue]);
        // Enqueue the second public call
        context.call_public_function(targetContract, targetSelector, [targetValue + 1]);
    }

    // Private function to enqueue a call to the pubEntryPoint function of this same contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueueCallToPubEntryPoint(
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) {
        let pubEntryPointSelector = compute_selector("pubEntryPoint(Field,Field,Field)");
        let thisAddress = context.this_address();
        let _void = context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);
    }

    // Private function to enqueue two calls to the pubEntryPoint function of this same contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueueCallsToPubEntryPoint(
        targetContract: Field,
        targetSelector: Field,
        targetValue: Field,
    ) {
        let pubEntryPointSelector = compute_selector("pubEntryPoint(Field,Field,Field)");
        let thisAddress = context.this_address();
        
        context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue]);

        context.call_public_function(thisAddress, pubEntryPointSelector, [targetContract, targetSelector, targetValue + 1]);
    }
}
