mod interfaces;

contract FPC {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::is_empty};
    use dep::aztec::state_vars::SharedImmutable;
    use dep::aztec::prelude::Deserialize;
    use dep::aztec::context::gas::GasOpts;
    use crate::interfaces::Token;

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

        let _res = Token::at(asset).unshield(
            &mut context,
            context.msg_sender(),
            context.this_address(),
            amount,
            nonce
        );

        let _void = context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("pay_fee_with_shielded_rebate(Field,(Field),Field)"),
            [amount, asset.to_field(), secret_hash]
        );
    }

    #[aztec(private)]
    fn fee_entrypoint_public(amount: Field, asset: AztecAddress, nonce: Field) {
        let _void = context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("prepare_fee((Field),Field,(Field),Field)"),
            [context.msg_sender().to_field(), amount, asset.to_field(), nonce]
        );

        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("pay_fee((Field),Field,(Field))"),
            [context.msg_sender().to_field(), amount, asset.to_field()]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn prepare_fee(from: AztecAddress, amount: Field, asset: AztecAddress, nonce: Field) {
        let _res = Token::at(asset).transfer_public(&mut context, from, context.this_address(), amount, nonce);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_fee(refund_address: AztecAddress, amount: Field, asset: AztecAddress) {
        let refund: Field = context.call_public_function(
            storage.gas_token_address.read_public(),
            FunctionSelector::from_signature("pay_fee(Field)"),
            [amount],
            GasOpts::default()
        ).deserialize_into();

        // Just do public refunds for the present
        Token::at(asset).transfer_public(&mut context, context.this_address(), refund_address, refund, 0)
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn pay_fee_with_shielded_rebate(amount: Field, asset: AztecAddress, secret_hash: Field) {
        let refund: Field = context.call_public_function(
            storage.gas_token_address.read_public(),
            FunctionSelector::from_signature("pay_fee(Field)"),
            [amount],
            GasOpts::default()
        ).deserialize_into();

        Token::at(asset).shield(&mut context, context.this_address(), refund, secret_hash, 0);
    }
}
