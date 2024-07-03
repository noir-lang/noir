mod types;
mod test;

// Minimal token implementation that supports `AuthWit` accounts and private refunds

contract PrivateToken {
    use dep::compressed_string::FieldCompressedString;
    use dep::aztec::{
        hash::compute_secret_hash,
        prelude::{NoteGetterOptions, Map, PublicMutable, SharedImmutable, PrivateSet, AztecAddress},
        protocol_types::{
        abis::function_selector::FunctionSelector, hash::pedersen_hash,
        constants::GENERATOR_INDEX__INNER_NOTE_HASH
    },
        oracle::unsafe_rand::unsafe_rand,
        encrypted_logs::encrypted_note_emission::{encode_and_encrypt_note, encode_and_encrypt_note_with_keys}
    };
    use dep::authwit::{auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public}};
    use crate::types::{token_note::{TokenNote, TOKEN_NOTE_LEN}, balances_map::BalancesMap};
    use dep::std::embedded_curve_ops::EmbeddedCurvePoint;
    use dep::std::ec::tecurve::affine::Point;

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
        fee_payer_npk_m_hash: Field,
        sponsored_user: AztecAddress,
        funded_amount: Field,
        refund_nonce: Field
    ) {
        assert_current_call_valid_authwit(&mut context, sponsored_user);
        let header = context.get_header();
        let sponsored_user_npk_m_hash = header.get_npk_m_hash(&mut context, sponsored_user);
        let sponsored_user_ovpk = header.get_ovpk_m(&mut context, sponsored_user);
        let sponsored_user_ivpk = header.get_ivpk_m(&mut context, sponsored_user);
        storage.balances.sub(sponsored_user_npk_m_hash, U128::from_integer(funded_amount)).emit(encode_and_encrypt_note_with_keys(&mut context, sponsored_user_ovpk, sponsored_user_ivpk));
        let points = TokenNote::generate_refund_points(
            fee_payer_npk_m_hash,
            sponsored_user_npk_m_hash,
            funded_amount,
            refund_nonce
        );
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("complete_refund(Field,Field,Field,Field)"),
            [points[0].x, points[0].y, points[1].x, points[1].y]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn complete_refund(
        fpc_point_x: Field,
        fpc_point_y: Field,
        user_point_x: Field,
        user_point_y: Field
    ) {
        let fpc_point = EmbeddedCurvePoint { x: fpc_point_x, y: fpc_point_y, is_infinite: false };
        let user_point = EmbeddedCurvePoint { x: user_point_x, y: user_point_y, is_infinite: false };
        let tx_fee = context.transaction_fee();
        let note_hashes = TokenNote::complete_refund(fpc_point, user_point, tx_fee);

        // `compute_inner_note_hash` manually, without constructing the note
        // `3` is the storage slot of the balances
        context.push_note_hash(
            pedersen_hash(
                [PrivateToken::storage().balances.slot, note_hashes[0]],
                GENERATOR_INDEX__INNER_NOTE_HASH
            )
        );
        context.push_note_hash(
            pedersen_hash(
                [PrivateToken::storage().balances.slot, note_hashes[1]],
                GENERATOR_INDEX__INNER_NOTE_HASH
            )
        );
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
