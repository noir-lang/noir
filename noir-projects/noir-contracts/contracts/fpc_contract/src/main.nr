mod interfaces;

contract FPC {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
    use dep::aztec::state_vars::SharedImmutable;

    use crate::interfaces::Token;

    struct Storage {
        other_asset: SharedImmutable<AztecAddress>,
        fee_asset: SharedImmutable<AztecAddress>,
    }

    #[aztec(private)]
    fn constructor(other_asset: AztecAddress, fee_asset: AztecAddress) {
        let selector = FunctionSelector::from_signature("_initialize((Field),(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [other_asset.to_field(), fee_asset.to_field()]
        );
    }

    #[aztec(public)]
    internal fn _initialize(other_asset: AztecAddress, fee_asset: AztecAddress) {
        storage.other_asset.initialize(other_asset);
        storage.fee_asset.initialize(fee_asset);
    }

    #[aztec(private)]
    fn fee_entrypoint_private(amount: Field, asset: AztecAddress, nonce: Field) {
        assert(asset == storage.other_asset.read_private());

        let _res = Token::at(asset).unshield(
            &mut context,
            context.msg_sender(),
            context.this_address(),
            amount,
            nonce
        );

        let _void = context.call_public_function(
            storage.fee_asset.read_private(),
            FunctionSelector::from_signature("pay_fee(Field)"),
            [amount.to_field()]
        );
    }

    #[aztec(private)]
    fn fee_entrypoint_public(amount: Field, asset: AztecAddress, nonce: Field) {
        let _void = context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("prepare_fee((Field),Field,(Field),Field)"),
            [context.msg_sender().to_field(), amount, asset.to_field(), nonce]
        );

        let _void = context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("pay_fee((Field),Field,(Field))"),
            [context.msg_sender().to_field(), amount, asset.to_field()]
        );
    }

    #[aztec(public)]
    internal fn prepare_fee(from: AztecAddress, amount: Field, asset: AztecAddress, nonce: Field) {
        let _res = Token::at(asset).transfer_public(context, from, context.this_address(), amount, nonce);
    }

    #[aztec(public)]
    internal fn pay_fee(from: AztecAddress, amount: Field, asset: AztecAddress) {
        let _void = context.call_public_function(
            storage.fee_asset.read_public(),
            FunctionSelector::from_signature("pay_fee(Field)"),
            [amount.to_field()]
        );
        // TODO handle refunds
    }
}
