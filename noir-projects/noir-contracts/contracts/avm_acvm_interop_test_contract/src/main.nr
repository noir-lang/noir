use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

contract AvmAcvmInteropTest {
    // Libs
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::aztec::context::gas::GasOpts;
    use dep::aztec::context::public_context::FunctionReturns;
    use dep::aztec::protocol_types::abis::function_selector::FunctionSelector;
    use dep::authwit::{auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public}};

    // Use the standard context interface to emit a new nullifier
    #[aztec(public-vm)]
    fn new_nullifier(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
    }

    /************************************************************************
     * ACVM interoperability
     ************************************************************************/
    #[aztec(public)]
    fn constant_field_acvm() -> pub Field {
        123456
    }

    #[aztec(public-vm)]
    fn constant_field_avm() -> pub Field {
        123456
    }

    #[aztec(public)]
    fn new_nullifier_acvm(nullifier: Field) -> pub Field {
        context.push_new_nullifier(nullifier, 0);
    }

    #[aztec(public)]
    fn assert_unsiloed_nullifier_acvm(unsiloed_nullifier: Field) {
        assert(context.nullifier_exists(unsiloed_nullifier, context.this_address()));
    }

    #[aztec(public-vm)]
    fn call_acvm_from_avm() -> pub Field {
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("constant_field_acvm()"),
            [],
            GasOpts::default()
        ).deserialize_into()
    }

    #[aztec(public)]
    fn call_avm_from_acvm() -> pub Field {
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("constant_field_avm()"),
            [],
            GasOpts::default()
        ).deserialize_into()
    }

    #[aztec(public-vm)]
    fn avm_to_acvm_call(selector: FunctionSelector, args: Field) {
        context.call_public_function(context.this_address(), selector, [args], GasOpts::default()).assert_empty();
    }

    /************************************************************************
     * Authwit functions
     ************************************************************************/
    #[aztec(public-vm)]
    fn test_authwit_send_money(from: AztecAddress, _to: AztecAddress, _amount: Field) {
        assert_current_call_valid_authwit_public(&mut context, from);
    }
}
