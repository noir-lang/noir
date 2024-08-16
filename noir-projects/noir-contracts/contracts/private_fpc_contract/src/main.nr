mod settings;

contract PrivateFPC {
    use dep::aztec::{protocol_types::{address::AztecAddress, hash::compute_siloed_nullifier}, state_vars::SharedImmutable};
    use dep::token::Token;
    use crate::settings::Settings;

    #[aztec(storage)]
    struct Storage {
        settings: SharedImmutable<Settings>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(other_asset: AztecAddress, admin: AztecAddress) {
        let settings = Settings { other_asset, admin };
        storage.settings.initialize(settings);
    }

    #[aztec(private)]
    fn fund_transaction_privately(amount: Field, asset: AztecAddress, user_randomness: Field) {
        // TODO: Once SharedImmutable performs only 1 merkle proof here, we'll save ~4k gates
        let settings = storage.settings.read_private();

        assert(asset == settings.other_asset);

        // We use different randomness for fee payer to prevent a potential privacy leak (see description
        // of `setup_refund(...)` function in Token for details.
        let fee_payer_randomness = compute_siloed_nullifier(context.this_address(), user_randomness);
        // We emit fee payer randomness as nullifier to ensure FPC admin can reconstruct their fee note - note that
        // protocol circuits will perform the siloing as was done above and hence the final nullifier will be correct
        // fee payer randomness.
        context.push_nullifier(user_randomness);

        Token::at(asset).setup_refund(
            settings.admin,
            context.msg_sender(),
            amount,
            user_randomness,
            fee_payer_randomness
        ).call(&mut context);
        context.set_as_fee_payer();
    }
}
