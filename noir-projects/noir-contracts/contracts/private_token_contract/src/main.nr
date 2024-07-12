mod types;
mod test;

// Minimal token implementation that supports `AuthWit` accounts and private refunds

contract PrivateToken {
    use dep::compressed_string::FieldCompressedString;
    use dep::aztec::{
        note::utils::compute_inner_note_hash_from_preimage, hash::compute_secret_hash,
        prelude::{NoteGetterOptions, Map, PublicMutable, SharedImmutable, PrivateSet, AztecAddress},
        protocol_types::{abis::function_selector::FunctionSelector, point::Point, hash::pedersen_hash},
        oracle::unsafe_rand::unsafe_rand,
        encrypted_logs::encrypted_note_emission::{encode_and_encrypt_note, encode_and_encrypt_note_with_keys}
    };
    use dep::authwit::{auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public}};
    use crate::types::{token_note::{TokenNote, TOKEN_NOTE_LEN}, balances_map::BalancesMap};

    #[aztec(storage)]
    struct Storage {
        admin: PublicMutable<AztecAddress>,
        minters: Map<AztecAddress, PublicMutable<bool>>,
        balances: BalancesMap<TokenNote>,
        total_supply: PublicMutable<U128>,
        symbol: SharedImmutable<FieldCompressedString>,
        name: SharedImmutable<FieldCompressedString>,
        decimals: SharedImmutable<u8>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress, name: str<31>, symbol: str<31>, decimals: u8) {
        assert(!admin.is_zero(), "invalid admin");
        storage.admin.write(admin);
        storage.minters.at(admin).write(true);
        storage.name.initialize(FieldCompressedString::from_string(name));
        storage.symbol.initialize(FieldCompressedString::from_string(symbol));
        storage.decimals.initialize(decimals);
    }

    #[aztec(public)]
    fn set_admin(new_admin: AztecAddress) {
        assert(storage.admin.read().eq(context.msg_sender()), "caller is not admin");
        storage.admin.write(new_admin);
    }

    #[aztec(public)]
    fn public_get_name() -> pub FieldCompressedString {
        storage.name.read_public()
    }

    #[aztec(private)]
    fn private_get_name() -> pub FieldCompressedString {
        storage.name.read_private()
    }

    unconstrained fn un_get_name() -> pub [u8; 31] {
        storage.name.read_public().to_bytes()
    }

    #[aztec(public)]
    fn public_get_symbol() -> pub FieldCompressedString {
        storage.symbol.read_public()
    }

    #[aztec(private)]
    fn private_get_symbol() -> pub FieldCompressedString {
        storage.symbol.read_private()
    }

    unconstrained fn un_get_symbol() -> pub [u8; 31] {
        storage.symbol.read_public().to_bytes()
    }

    #[aztec(public)]
    fn public_get_decimals() -> pub u8 {
        storage.decimals.read_public()
    }

    #[aztec(private)]
    fn private_get_decimals() -> pub u8 {
        storage.decimals.read_private()
    }

    unconstrained fn un_get_decimals() -> pub u8 {
        storage.decimals.read_public()
    }

    #[aztec(public)]
    fn set_minter(minter: AztecAddress, approve: bool) {
        assert(storage.admin.read().eq(context.msg_sender()), "caller is not admin");
        storage.minters.at(minter).write(approve);
    }

    #[aztec(private)]
    fn privately_mint_private_note(amount: Field) {
        let caller = context.msg_sender();
        let header = context.get_header();
        let caller_npk_m_hash = header.get_npk_m_hash(&mut context, caller);
        storage.balances.add(caller_npk_m_hash, U128::from_integer(amount)).emit(encode_and_encrypt_note(&mut context, caller, caller));
        PrivateToken::at(context.this_address()).assert_minter_and_mint(context.msg_sender(), amount).enqueue(&mut context);
    }

    #[aztec(public)]
    fn assert_minter_and_mint(minter: AztecAddress, amount: Field) {
        assert(storage.minters.at(minter).read(), "caller is not minter");
        let supply = storage.total_supply.read() + U128::from_integer(amount);
        storage.total_supply.write(supply);
    }

    #[aztec(private)]
    fn transfer_from(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let header = context.get_header();
        let from_ovpk = header.get_ovpk_m(&mut context, from);
        let from_ivpk = header.get_ivpk_m(&mut context, from);
        let from_npk_m_hash = header.get_npk_m_hash(&mut context, from);
        let to_ivpk = header.get_ivpk_m(&mut context, to);
        let to_npk_m_hash = header.get_npk_m_hash(&mut context, to);

        let amount = U128::from_integer(amount);
        storage.balances.sub(from_npk_m_hash, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, from_ivpk));
        storage.balances.add(to_npk_m_hash, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, to_ivpk));
    }

    #[aztec(private)]
    fn transfer(to: AztecAddress, amount: Field) {
        let from = context.msg_sender();
        let header = context.get_header();
        let from_ovpk = header.get_ovpk_m(&mut context, from);
        let from_ivpk = header.get_ivpk_m(&mut context, from);
        let from_npk_m_hash = header.get_npk_m_hash(&mut context, from);
        let to_ivpk = header.get_ivpk_m(&mut context, to);
        let to_npk_m_hash = header.get_npk_m_hash(&mut context, to);

        let amount = U128::from_integer(amount);
        storage.balances.sub(from_npk_m_hash, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, from_ivpk));
        storage.balances.add(to_npk_m_hash, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, to_ivpk));
    }

    #[aztec(private)]
    fn balance_of_private(owner: AztecAddress) -> pub Field {
        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);
        storage.balances.to_unconstrained().balance_of(owner_npk_m_hash).to_integer()
    }

    unconstrained fn balance_of_unconstrained(owner_npk_m_hash: Field) -> pub Field {
        storage.balances.balance_of(owner_npk_m_hash).to_integer()
    }

    #[aztec(private)]
    fn setup_refund(
        fee_payer_npk_m_hash: Field, // NpkMHash of the entity which will receive the fee note.
        user: AztecAddress, // A user for which we are setting up the fee refund.
        funded_amount: Field, // The amount the user funded the fee payer with (represents fee limit).
        user_randomness: Field, // A randomness to mix in with the generated refund note for the sponsored user.
        fee_payer_randomness: Field // A randomness to mix in with the generated fee note for the fee payer.
    ) {
        // 1. This function is called by fee paying contract (fee_payer) when setting up a refund so we need to support
        // the authwit flow here and check that the user really permitted fee_payer to set up a refund on their behalf.
        assert_current_call_valid_authwit(&mut context, user);

        // 2. Get all the relevant user keys
        let header = context.get_header();
        let user_npk_m_hash = header.get_npk_m_hash(&mut context, user);
        let user_ovpk = header.get_ovpk_m(&mut context, user);
        let user_ivpk = header.get_ivpk_m(&mut context, user);

        // 3. Deduct the funded amount from the user's balance - this is a maximum fee a user is willing to pay
        // (called fee limit in aztec spec). The difference between fee limit and the actual tx fee will be refunded 
        // to the user in the `complete_refund(...)` function.
        // TODO(#7324), TODO(#7323): using npk_m_hash here is vulnerable in 2 ways described in the linked issues.
        storage.balances.sub(user_npk_m_hash, U128::from_integer(funded_amount)).emit(encode_and_encrypt_note_with_keys(&mut context, user_ovpk, user_ivpk));

        // 4. We generate the refund points.
        let (fee_payer_point, user_point) = TokenNote::generate_refund_points(
            fee_payer_npk_m_hash,
            user_npk_m_hash,
            funded_amount,
            user_randomness,
            fee_payer_randomness
        );

        // 5. Set the public teardown function to `complete_refund(...)`. Public teardown is the only time when a public
        // function has access to the final transaction fee, which is needed to compute the actual refund amount.
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("complete_refund((Field,Field,bool),(Field,Field,bool))"),
            [
            fee_payer_point.x, fee_payer_point.y, fee_payer_point.is_infinite as Field, user_point.x, user_point.y, user_point.is_infinite as Field
        ]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn complete_refund(fee_payer_point: Point, user_point: Point) {
        // 1. We get the final note content hashes by calling the `complete_refund` on the note.
        // We use 1:1 exchange rate between fee juice and token. So using `tx_fee` is enough
        let tx_fee = context.transaction_fee();
        let (fee_payer_note_content_hash, user_note_content_hash) = TokenNote::complete_refund(fee_payer_point, user_point, tx_fee);

        // 2. Now we "manually" compute the inner note hashes.
        let fee_payer_inner_note_hash = compute_inner_note_hash_from_preimage(
            PrivateToken::storage().balances.slot,
            fee_payer_note_content_hash
        );
        let user_inner_note_hash = compute_inner_note_hash_from_preimage(PrivateToken::storage().balances.slot, user_note_content_hash);

        // 3. At last we emit the note hashes.
        context.push_note_hash(fee_payer_inner_note_hash);
        context.push_note_hash(user_inner_note_hash);
        // --> Once the tx is settled user and fee recipient can add the notes to their pixies.
    }

    /// Internal ///

    #[aztec(public)]
    #[aztec(internal)]
    fn _reduce_total_supply(amount: Field) {
        // Only to be called from burn.
        let new_supply = storage.total_supply.read().sub(U128::from_integer(amount));
        storage.total_supply.write(new_supply);
    }

    /// Unconstrained ///

    unconstrained fn admin() -> pub Field {
        storage.admin.read().to_field()
    }

    unconstrained fn is_minter(minter: AztecAddress) -> pub bool {
        storage.minters.at(minter).read()
    }

    unconstrained fn total_supply() -> pub Field {
        storage.total_supply.read().to_integer()
    }
}
