use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

contract AvmNestedCallsTest {
    // Libs
    use dep::aztec::state_vars::PublicMutable;
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::aztec::context::gas::GasOpts;
    use dep::aztec::context::public_context::FunctionReturns;
    use dep::aztec::protocol_types::abis::function_selector::FunctionSelector;

    #[aztec(storage)]
    struct Storage {
        single: PublicMutable<Field>,
    }

    /************************************************************************
     * Storage
     ************************************************************************/
    #[aztec(public-vm)]
    fn set_storage_single(a: Field) {
        storage.single.write(a);
    }

    #[aztec(public-vm)]
    fn add_args_return(arg_a: Field, arg_b: Field) -> pub Field {
        arg_a + arg_b
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public-vm)]
    fn new_nullifier(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
    }

    // Directly call the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)").to_field();

        // Nested call
        let results = context.call_public_function_raw(
            GasOpts::default(),
            context.this_address(),
            selector,
            [arg_a, arg_b].as_slice()
        );
        let data_to_return: [Field; 1] = results.0;
        // this explicit size is necessary to ensure that ret_size is compile-time
        // (ensure the data_to_return is in a HeapArray not a HeapVector)
        let success: u8 = results.1;

        assert(success == 1, "Call failed");

        let add_result = data_to_return[0];
        add_result
    }

    // Directly call the external call opcode to initiate a nested call to the add function with user-specified gas
    #[aztec(public-vm)]
    fn raw_nested_call_to_add_with_gas(
        arg_a: Field,
        arg_b: Field,
        l1_gas: Field,
        l2_gas: Field,
        da_gas: Field
    ) -> pub Field {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)").to_field();

        // Nested call
        let results = context.call_public_function_raw(
            GasOpts::new(l1_gas, l2_gas, da_gas),
            context.this_address(),
            selector,
            [arg_a, arg_b].as_slice()
        );
        let data_to_return: [Field; 1] = results.0;
        // this explicit size is necessary to ensure that ret_size is compile-time
        // (ensure the data_to_return is in a HeapArray not a HeapVector)
        let _success: u8 = results.1;

        let add_result = data_to_return[0];
        add_result
    }

    // Use the `call_public_function` wrapper to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmNestedCallsTest::at(context.this_address()).add_args_return(arg_a, arg_b).call(&mut context, GasOpts::default())
    }

    // Directly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub (Field, u8) {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)").to_field();

        let (result_data, success): ([Field; 1], u8) = context.static_call_public_function_raw(
            GasOpts::default(),
            context.this_address(),
            selector,
            [arg_a, arg_b].as_slice()
        );

        (result_data[0], success)
    }

    // Directly call_static `set_storage_single`. Should fail since it's accessing storage.
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_set_storage() -> pub u8 {
        let selector = FunctionSelector::from_signature("set_storage_single(Field)").to_field();
        let calldata: [Field; 1] = [20];

        let (_data_to_return, success): ([Field; 0], u8) = context.static_call_public_function_raw(
            GasOpts::default(),
            context.this_address(),
            selector,
            calldata.as_slice()
        );

        success
    }

    // Indirectly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmNestedCallsTest::at(context.this_address()).add_args_return(arg_a, arg_b).static_call(&mut context, GasOpts::default())
    }

    // Indirectly call_static `set_storage_single`. Should revert since it's accessing storage.
    #[aztec(public-vm)]
    fn nested_static_call_to_set_storage() {
        AvmNestedCallsTest::at(context.this_address()).set_storage_single(20).static_call(&mut context, GasOpts::default());
    }

    #[aztec(public-vm)]
    fn create_same_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        AvmNestedCallsTest::at(nestedAddress).new_nullifier(nullifier).call(&mut context, GasOpts::default());
    }

    #[aztec(public-vm)]
    fn create_different_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        AvmNestedCallsTest::at(nestedAddress).new_nullifier(nullifier + 1).call(&mut context, GasOpts::default());
    }
}
