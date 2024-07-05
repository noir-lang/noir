mod lib;

contract PrivateFPC {
    use dep::aztec::protocol_types::{abis::log_hash::LogHash, address::AztecAddress, hash::poseidon2_hash};
    use dep::aztec::state_vars::SharedImmutable;
    use dep::private_token::PrivateToken;
    use crate::lib::emit_randomness_as_unencrypted_log;

    #[aztec(storage)]
    struct Storage {
        other_asset: SharedImmutable<AztecAddress>,
        admin_npk_m_hash: SharedImmutable<Field>
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(other_asset: AztecAddress, admin_npk_m_hash: Field) {
        storage.other_asset.initialize(other_asset);
        storage.admin_npk_m_hash.initialize(admin_npk_m_hash);
    }

    #[aztec(private)]
    fn fund_transaction_privately(amount: Field, asset: AztecAddress, user_randomness: Field) {
        assert(asset == storage.other_asset.read_private());
        // convince the FPC we are not cheating
        context.push_nullifier(user_randomness, 0);

        // We use different randomness for fee payer to prevent a potential privay leak (see impl
        // of PrivatelyRefundable for TokenNote for details).
        let fee_payer_randomness = poseidon2_hash([user_randomness]);
        // We emit fee payer randomness to ensure FPC admin can reconstruct their fee note
        emit_randomness_as_unencrypted_log(&mut context, fee_payer_randomness);

        PrivateToken::at(asset).setup_refund(
            storage.admin_npk_m_hash.read_private(),
            context.msg_sender(),
            amount,
            user_randomness,
            fee_payer_randomness
        ).call(&mut context);
        context.set_as_fee_payer();
    }
}
