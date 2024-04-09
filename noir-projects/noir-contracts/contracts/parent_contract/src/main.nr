// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector};
    use dep::aztec::context::gas::GasOpts;

    // Private function to call another private function in the target_contract using the provided selector
    #[aztec(private)]
    fn entry_point(target_contract: AztecAddress, target_selector: FunctionSelector) -> Field {
        // Call the target private function
        let return_values = context.call_private_function(target_contract, target_selector, [0]);

        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    // Public function to directly call another public function to the target_contract using the selector and value provided
    #[aztec(public)]
    fn pub_entry_point(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        init_value: Field
    ) -> Field {
        let return_values = context.call_public_function(
            target_contract,
            target_selector,
            [init_value],
            GasOpts::default()
        );

        return_values[0]
    }

    // Same as pub_entry_point, but calls the target contract twice, using the return value from the first invocation as the argument for the second.
    #[aztec(public)]
    fn pub_entry_point_twice(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        init_value: Field
    ) -> Field {
        let return_value = context.call_public_function(
            target_contract,
            target_selector,
            [init_value],
            GasOpts::default()
        )[0];
        let return_values = context.call_public_function(
            target_contract,
            target_selector,
            [return_value],
            GasOpts::default()
        );

        return_values[0]
    }

    // Private function to enqueue a call to the target_contract address using the selector and argument provided
    #[aztec(private)]
    fn enqueue_call_to_child(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        target_value: Field
    ) {
        context.call_public_function(target_contract, target_selector, [target_value]);
    }

    // Private function that enqueues two calls to a child contract: 
    // - one through a nested call to enqueue_call_to_child with value 10,
    // - followed by one issued directly from this function with value 20.
    #[aztec(private)]
    fn enqueue_calls_to_child_with_nested_first(
        target_contract: AztecAddress,
        target_selector: FunctionSelector
    ) {
        let enqueue_call_to_child_selector = FunctionSelector::from_signature("enqueue_call_to_child((Field),(u32),Field)");
        let _ret = context.call_private_function(
            context.this_address(),
            enqueue_call_to_child_selector,
            [target_contract.to_field(), target_selector.to_field(), 10]
        );
        context.call_public_function(target_contract, target_selector, [20]);
    }

    // Private function that enqueues two calls to a child contract: 
    // - one issued directly from this function with value 20,
    // - followed by one through a nested call to enqueue_call_to_child with value 10.
    #[aztec(private)]
    fn enqueue_calls_to_child_with_nested_last(
        target_contract: AztecAddress,
        target_selector: FunctionSelector
    ) {
        context.call_public_function(target_contract, target_selector, [20]);
        let enqueue_call_to_child_selector = FunctionSelector::from_signature("enqueue_call_to_child((Field),(u32),Field)");
        let _ret = context.call_private_function(
            context.this_address(),
            enqueue_call_to_child_selector,
            [target_contract.to_field(), target_selector.to_field(), 10]
        );
    }

    // Private function to enqueue a call to the target_contract address using the selector and argument provided
    #[aztec(private)]
    fn enqueue_call_to_child_twice(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        target_value: Field
    ) {
        // Enqueue the first public call
        context.call_public_function(target_contract, target_selector, [target_value]);
        // Enqueue the second public call
        context.call_public_function(target_contract, target_selector, [target_value + 1]);
    }

    // Private function to enqueue a call to the pub_entry_point function of this same contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueue_call_to_pub_entry_point(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        target_value: Field
    ) {
        let pub_entry_point_selector = FunctionSelector::from_signature("pub_entry_point((Field),(u32),Field)");
        let this_address = context.this_address();
        let _void = context.call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), target_value]
        );
    }

    // Private function to enqueue two calls to the pub_entry_point function of this same contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueue_calls_to_pub_entry_point(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        target_value: Field
    ) {
        let pub_entry_point_selector = FunctionSelector::from_signature("pub_entry_point((Field),(u32),Field)");
        let this_address = context.this_address();

        context.call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), target_value]
        );

        context.call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), target_value + 1]
        );
    }

    #[aztec(private)]
    fn private_static_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 2]
    ) -> Field {
        // Call the target private function
        let return_values = context.static_call_private_function(target_contract, target_selector, args);

        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    #[aztec(private)]
    fn private_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 2]
    ) -> Field {
        // Call the target private function
        let return_values = context.call_private_function(target_contract, target_selector, args);

        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    // Private function to set a static context and verify correct propagation for nested private calls
    #[aztec(private)]
    fn private_nested_static_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 2]
    ) -> Field {
        // Call the target private function statically
        let private_call_selector = FunctionSelector::from_signature("private_call((Field),(u32),[Field;2])");
        let this_address = context.this_address();
        let return_values = context.static_call_private_function(
            this_address,
            private_call_selector,
            [target_contract.to_field(), target_selector.to_field(), args[0], args[1]]
        );

        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    // Public function to directly call another public function to the target_contract using the selector and value provided
    #[aztec(public)]
    fn public_static_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) -> Field {
        let return_values = context.static_call_public_function(target_contract, target_selector, args, GasOpts::default());

        return_values[0]
    }

    // Public function to set a static context and verify correct propagation for nested public calls
    #[aztec(public)]
    fn public_nested_static_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) -> Field {
        // Call the target public function through the pub entrypoint statically
        let pub_entry_point_selector = FunctionSelector::from_signature("pub_entry_point((Field),(u32),Field)");
        let this_address = context.this_address();
        let return_values = context.static_call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), args[0]],
            GasOpts::default()
        );
        return_values[0]
    }

    // Private function to enqueue a static call to the pub_entry_point function of another contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueue_static_nested_call_to_pub_function(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) {
        // Call the target public function through the pub entrypoint statically
        let pub_entry_point_selector = FunctionSelector::from_signature("pub_entry_point((Field),(u32),Field)");
        let this_address = context.this_address();
        context.static_call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), args[0]]
        );
    }

    // Private function to enqueue a static call to the pub_entry_point function of another contract, passing the target arguments provided
    #[aztec(private)]
    fn enqueue_static_call_to_pub_function(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) {
        // Call the target private function
        context.static_call_public_function(target_contract, target_selector, args);
    }
}
