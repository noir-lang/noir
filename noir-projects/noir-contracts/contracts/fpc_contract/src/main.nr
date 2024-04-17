contract FPC {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::is_empty};
    use dep::aztec::state_vars::SharedImmutable;
    use dep::token::Token;
    use dep::gas_token::GasToken;
    use dep::aztec::context::gas::GasOpts;

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
        FPC::at(context.this_address()).pay_fee_with_shielded_rebate(amount, asset, secret_hash).enqueue(&mut context);
    }

    #[aztec(private)]
    fn fee_entrypoint_public(amount: Field, asset: AztecAddress, nonce: Field) {
        FPC::at(context.this_address()).prepare_fee(context.msg_sender(), amount, asset, nonce).enqueue(&mut context);
        FPC::at(context.this_address()).pay_fee(context.msg_sender(), amount, asset).enqueue(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn prepare_fee(from: AztecAddress, amount: Field, asset: AztecAddress, nonce: Field) {
        Token::at(asset).transfer_public(from, context.this_address(), amount, nonce).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_fee(refund_address: AztecAddress, amount: Field, asset: AztecAddress) {
        let refund = GasToken::at(storage.gas_token_address.read_public()).pay_fee(amount).call(&mut context);
        // Just do public refunds for the present
        Token::at(asset).transfer_public(context.this_address(), refund_address, refund, 0).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_fee_with_shielded_rebate(amount: Field, asset: AztecAddress, secret_hash: Field) {
        let refund = GasToken::at(storage.gas_token_address.read_public()).pay_fee(amount).call(&mut context);
        Token::at(asset).shield(context.this_address(), refund, secret_hash, 0).call(&mut context);
    }
}
