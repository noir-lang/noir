
// A contract used along with `Child` contract to test nested calls.
contract Parent {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, Deserialize};
    use dep::aztec::context::gas::GasOpts;
    // Private function to call another private function in the target_contract using the provided selector
    #[aztec(private)]
    fn entry_point(target_contract: AztecAddress, target_selector: FunctionSelector) -> Field {
        // Call the target private function
        context.call_private_function(target_contract, target_selector, [0]).unpack_into()
    }

    // Public function to directly call another public function to the target_contract using the selector and value provided
    #[aztec(public)]
    fn pub_entry_point(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        init_value: Field
    ) -> Field {
        context.call_public_function(
            target_contract,
            target_selector,
            [init_value].as_slice(),
            GasOpts::default()
        ).deserialize_into()
    }

    // Same as pub_entry_point, but calls the target contract twice, using the return value from the first invocation as the argument for the second.
    #[aztec(public)]
    fn pub_entry_point_twice(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        init_value: Field
    ) -> Field {
        let return_value: Field = context.call_public_function(
            target_contract,
            target_selector,
            [init_value].as_slice(),
            GasOpts::default()
        ).deserialize_into();
        context.call_public_function(
            target_contract,
            target_selector,
            [return_value].as_slice(),
            GasOpts::default()
        ).deserialize_into()
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
        context.static_call_private_function(target_contract, target_selector, args).unpack_into()
    }

    #[aztec(private)]
    fn private_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 2]
    ) -> Field {
        // Call the target private function
        context.call_private_function(target_contract, target_selector, args).unpack_into()
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
        let return_value: Field = context.static_call_private_function(
            this_address,
            private_call_selector,
            [target_contract.to_field(), target_selector.to_field(), args[0], args[1]]
        ).unpack_into();

        // Copy the return value from the call to this function's return values
        return_value
    }

    // Public function to directly call another public function to the target_contract using the selector and value provided
    #[aztec(public)]
    fn public_static_call(
        target_contract: AztecAddress,
        target_selector: FunctionSelector,
        args: [Field; 1]
    ) -> Field {
        context.static_call_public_function(
            target_contract,
            target_selector,
            args.as_slice(),
            GasOpts::default()
        ).deserialize_into()
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
        context.static_call_public_function(
            this_address,
            pub_entry_point_selector,
            [target_contract.to_field(), target_selector.to_field(), args[0]].as_slice(),
            GasOpts::default()
        ).deserialize_into()
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

    use dep::aztec::test::{helpers::{cheatcodes, test_environment::TestEnvironment}};
    use dep::aztec::protocol_types::storage::map::derive_storage_slot_in_map;
    use dep::aztec::note::note_getter::{MAX_NOTES_PER_PAGE, view_notes};
    use dep::aztec::note::note_viewer_options::NoteViewerOptions;
    use dep::child_contract::Child;
    use dep::value_note::value_note::ValueNote;

    #[test]
    fn test_private_call() {
        // Setup env, generate keys
        let mut env = TestEnvironment::new();
        let owner = env.create_account();

        // Deploy child contract
        let child_contract = env.deploy("./@child_contract", "Child").without_initializer();
        let child_contract_address = child_contract.to_address();
        cheatcodes::advance_blocks_by(1);

        // Set value in child through parent
        let value_to_set = 7;
        let parent_private_set_call_interface = Parent::interface().private_call(
            child_contract_address,
            FunctionSelector::from_signature("private_set_value(Field,(Field))"),
            [value_to_set, owner.to_field()]
        );
        let result: Field = env.call_private(parent_private_set_call_interface);
        assert(result == value_to_set);
        // Read the stored value in the note. We have to change the contract address to the child contract in order to read its notes
        env.impersonate(child_contract_address);
        let counter_slot = Child::storage().a_map_with_private_values.slot;
        let owner_slot = derive_storage_slot_in_map(counter_slot, owner);
        let mut options = NoteViewerOptions::new();
        let notes: BoundedVec<ValueNote, MAX_NOTES_PER_PAGE> = view_notes(owner_slot, options);
        let note_value = notes.get(0).value;
        assert(note_value == value_to_set);
        assert(note_value == result);
        // Get value from child through parent
        let parent_private_get_call_interface = Parent::interface().private_call(
            child_contract_address,
            FunctionSelector::from_signature("private_get_value(Field,(Field))"),
            [7, owner.to_field()]
        );
        let read_result: Field = env.call_private(parent_private_get_call_interface);
        assert(note_value == read_result);
    }
}
