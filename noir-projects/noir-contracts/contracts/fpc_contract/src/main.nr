mod lib;

contract FPC {
    use dep::aztec::{
        protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::is_empty},
        state_vars::SharedImmutable, context::{interface::PublicContextInterface, gas::GasOpts}
    };
    use dep::token::Token;
    use dep::gas_token::GasToken;
    use crate::lib::compute_rebate;

    #[aztec(storage)]
    struct Storage {
        other_asset: SharedImmutable<AztecAddress>,
        gas_token_address: SharedImmutable<AztecAddress>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(other_asset: AztecAddress, gas_token_address: AztecAddress) {
        storage.other_asset.initialize(other_asset);
        storage.gas_token_address.initialize(gas_token_address);
    }

    #[aztec(private)]
    fn fee_entrypoint_private(amount: Field, asset: AztecAddress, secret_hash: Field, nonce: Field) {
        assert(asset == storage.other_asset.read_private());
        Token::at(asset).unshield(context.msg_sender(), context.this_address(), amount, nonce).call(&mut context);
        context.set_as_fee_payer();
        // Would like to get back to
        // FPC::at(context.this_address()).pay_refund_with_shielded_rebate(amount, asset, secret_hash).set_public_teardown_function(&mut context);
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("pay_refund_with_shielded_rebate(Field,(Field),Field)"),
            [amount, asset.to_field(), secret_hash]
        );
    }

    #[aztec(private)]
    fn fee_entrypoint_public(amount: Field, asset: AztecAddress, nonce: Field) {
        FPC::at(context.this_address()).prepare_fee(context.msg_sender(), amount, asset, nonce).enqueue(&mut context);
        context.set_as_fee_payer();
        // TODO(#6277) for improving interface:
        // FPC::at(context.this_address()).pay_refund(context.msg_sender(), amount, asset).set_public_teardown_function(&mut context);
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("pay_refund((Field),Field,(Field))"),
            [context.msg_sender().to_field(), amount, asset.to_field()]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn prepare_fee(from: AztecAddress, amount: Field, asset: AztecAddress, nonce: Field) {
        // docs:start:public_call
        Token::at(asset).transfer_public(from, context.this_address(), amount, nonce).call(&mut context);
        // docs:end:public_call
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_refund(refund_address: AztecAddress, amount: Field, asset: AztecAddress) {
        // Just do public refunds for the present
        let refund = compute_rebate(context, amount);
        Token::at(asset).transfer_public(context.this_address(), refund_address, refund, 0).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_refund_with_shielded_rebate(amount: Field, asset: AztecAddress, secret_hash: Field) {
        let refund = compute_rebate(context, amount);
        Token::at(asset).shield(context.this_address(), refund, secret_hash, 0).call(&mut context);
    }
}
