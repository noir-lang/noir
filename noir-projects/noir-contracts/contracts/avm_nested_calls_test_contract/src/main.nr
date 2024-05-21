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
    #[aztec(public)]
    fn set_storage_single(a: Field) {
        storage.single.write(a);
    }

    #[aztec(public)]
    fn add_args_return(arg_a: Field, arg_b: Field) -> pub Field {
        arg_a + arg_b
    }

    #[aztec(public)]
    fn assert_same(arg_a: Field, arg_b: Field) -> pub Field {
        assert(arg_a == arg_b, "Values are not equal");
        1
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public)]
    fn new_nullifier(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
    }

    // External call opcode to initiate a nested call to the add function with user-specified gas
    #[aztec(public)]
    fn nested_call_to_add_with_gas(
        arg_a: Field,
        arg_b: Field,
        l2_gas: Field,
        da_gas: Field
    ) -> pub Field {
        AvmNestedCallsTest::at(context.this_address()).add_args_return(arg_a, arg_b).with_gas(GasOpts::new(l2_gas, da_gas)).call(&mut context)
    }

    // Use the `call_public_function` wrapper to initiate a nested call to the add function
    #[aztec(public)]
    fn nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmNestedCallsTest::at(context.this_address()).add_args_return(arg_a, arg_b).call(&mut context)
    }

    // Indirectly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public)]
    fn nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmNestedCallsTest::at(context.this_address()).add_args_return(arg_a, arg_b).view(&mut context)
    }

    // Indirectly call_static `set_storage_single`. Should revert since it's accessing storage.
    #[aztec(public)]
    fn nested_static_call_to_set_storage() {
        AvmNestedCallsTest::at(context.this_address()).set_storage_single(20).view(&mut context);
    }

    #[aztec(public)]
    fn create_same_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        AvmNestedCallsTest::at(nestedAddress).new_nullifier(nullifier).call(&mut context);
    }

    #[aztec(public)]
    fn create_different_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        AvmNestedCallsTest::at(nestedAddress).new_nullifier(nullifier + 1).call(&mut context);
    }
}
