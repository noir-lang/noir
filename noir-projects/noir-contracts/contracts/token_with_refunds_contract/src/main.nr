// docs:start:token_all
// docs:start:imports
mod types;
mod test;

// Copy of standard token contract enhanced with refund functionality.

contract TokenWithRefunds {
    // Libs

    use dep::compressed_string::FieldCompressedString;

    use dep::aztec::{
        hash::compute_secret_hash,
        prelude::{NoteGetterOptions, Map, PublicMutable, SharedImmutable, PrivateSet, AztecAddress},
        encrypted_logs::{
        encrypted_note_emission::{
        encode_and_encrypt_note, encode_and_encrypt_note_with_keys,
        encode_and_encrypt_note_with_keys_unconstrained
    },
        encrypted_event_emission::{encode_and_encrypt_event, encode_and_encrypt_event_with_keys_unconstrained}
    }
    };

    // docs:start:import_authwit
    use dep::authwit::auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public, compute_authwit_nullifier};
    // docs:end:import_authwit

    use crate::types::{transparent_note::TransparentNote, token_note::{TokenNote, TOKEN_NOTE_LEN}, balances_map::BalancesMap};
    // docs:end::imports

    // TODO(#7425): Rename back to `Transfer`
    #[aztec(event)]
    struct Transfer2 {
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
    }

    // docs:start:storage_struct
    #[aztec(storage)]
    struct Storage {
        // docs:start:storage_admin
        admin: PublicMutable<AztecAddress>,
        // docs:end:storage_admin
        // docs:start:storage_minters
        minters: Map<AztecAddress, PublicMutable<bool>>,
        // docs:end:storage_minters
        // docs:start:storage_balances
        balances: BalancesMap<TokenNote>,
        // docs:end:storage_balances
        total_supply: PublicMutable<U128>,
        // docs:start:storage_pending_shields
        pending_shields: PrivateSet<TransparentNote>,
        // docs:end:storage_pending_shields
        public_balances: Map<AztecAddress, PublicMutable<U128>>,
        symbol: SharedImmutable<FieldCompressedString>,
        name: SharedImmutable<FieldCompressedString>,
        // docs:start:storage_decimals
        decimals: SharedImmutable<u8>,
        // docs:end:storage_decimals
    }
    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress, name: str<31>, symbol: str<31>, decimals: u8) {
        assert(!admin.is_zero(), "invalid admin");
        storage.admin.write(admin);
        storage.minters.at(admin).write(true);
        storage.name.initialize(FieldCompressedString::from_string(name));
        storage.symbol.initialize(FieldCompressedString::from_string(symbol));
        // docs:start:initialize_decimals
        storage.decimals.initialize(decimals);
        // docs:end:initialize_decimals
    }
    // docs:end:constructor

    // docs:start:set_admin
    #[aztec(public)]
    fn set_admin(new_admin: AztecAddress) {
        assert(storage.admin.read().eq(context.msg_sender()), "caller is not admin");
        // docs:start:write_admin
        storage.admin.write(new_admin);
        // docs:end:write_admin
    }
    // docs:end:set_admin

    #[aztec(public)]
    #[aztec(view)]
    fn public_get_name() -> pub FieldCompressedString {
        storage.name.read_public()
    }

    #[aztec(private)]
    #[aztec(view)]
    fn private_get_name() -> pub FieldCompressedString {
        storage.name.read_private()
    }

    #[aztec(public)]
    #[aztec(view)]
    fn public_get_symbol() -> pub FieldCompressedString {
        storage.symbol.read_public()
    }

    #[aztec(private)]
    #[aztec(view)]
    fn private_get_symbol() -> pub FieldCompressedString {
        storage.symbol.read_private()
    }

    #[aztec(public)]
    #[aztec(view)]
    fn public_get_decimals() -> pub u8 {
        // docs:start:read_decimals_public
        storage.decimals.read_public()
        // docs:end:read_decimals_public
    }

    #[aztec(private)]
    #[aztec(view)]
    fn private_get_decimals() -> pub u8 {
        // docs:start:read_decimals_private
        storage.decimals.read_private()
        // docs:end:read_decimals_private
    }

    // docs:start:admin
    #[aztec(public)]
    #[aztec(view)]
    fn admin() -> Field {
        storage.admin.read().to_field()
    }
    // docs:end:admin

    // docs:start:is_minter
    #[aztec(public)]
    #[aztec(view)]
    fn is_minter(minter: AztecAddress) -> bool {
        storage.minters.at(minter).read()
    }
    // docs:end:is_minter

    // docs:start:total_supply
    #[aztec(public)]
    #[aztec(view)]
    fn total_supply() -> Field {
        storage.total_supply.read().to_integer()
    }
    // docs:end:total_supply

    // docs:start:balance_of_public
    #[aztec(public)]
    #[aztec(view)]
    fn balance_of_public(owner: AztecAddress) -> Field {
        storage.public_balances.at(owner).read().to_integer()
    }
    // docs:end:balance_of_public

    // docs:start:set_minter
    #[aztec(public)]
    fn set_minter(minter: AztecAddress, approve: bool) {
        // docs:start:read_admin
        assert(storage.admin.read().eq(context.msg_sender()), "caller is not admin");
        // docs:end:read_admin
        // docs:start:write_minter
        storage.minters.at(minter).write(approve);
        // docs:end:write_minter
    }
    // docs:end:set_minter

    // docs:start:mint_public
    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        // docs:start:read_minter
        assert(storage.minters.at(context.msg_sender()).read(), "caller is not minter");
        // docs:end:read_minter
        let amount = U128::from_integer(amount);
        let new_balance = storage.public_balances.at(to).read().add(amount);
        let supply = storage.total_supply.read().add(amount);

        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(supply);
    }
    // docs:end:mint_public

    // docs:start:mint_private
    #[aztec(public)]
    fn mint_private(amount: Field, secret_hash: Field) {
        assert(storage.minters.at(context.msg_sender()).read(), "caller is not minter");
        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount, secret_hash);
        let supply = storage.total_supply.read().add(U128::from_integer(amount));

        storage.total_supply.write(supply);
        // docs:start:insert_from_public
        pending_shields.insert_from_public(&mut note);
        // docs:end:insert_from_public
    }
    // docs:end:mint_private

    // TODO: Nuke this - test functions do not belong to token contract!
    #[aztec(private)]
    fn privately_mint_private_note(amount: Field) {
        let caller = context.msg_sender();
        storage.balances.add(caller, U128::from_integer(amount)).emit(encode_and_encrypt_note(&mut context, caller, caller));

        TokenWithRefunds::at(context.this_address()).assert_minter_and_mint(context.msg_sender(), amount).enqueue(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn assert_minter_and_mint(minter: AztecAddress, amount: Field) {
        assert(storage.minters.at(minter).read(), "caller is not minter");
        let supply = storage.total_supply.read() + U128::from_integer(amount);
        storage.total_supply.write(supply);
    }

    // docs:start:shield
    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            // The redeem is only spendable once, so we need to ensure that you cannot insert multiple shields from the same message.
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = U128::from_integer(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount.to_field(), secret_hash);

        storage.public_balances.at(from).write(from_balance);
        pending_shields.insert_from_public(&mut note);
    }
    // docs:end:shield

    // docs:start:transfer_public
    #[aztec(public)]
    fn transfer_public(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = U128::from_integer(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);
        storage.public_balances.at(from).write(from_balance);

        let to_balance = storage.public_balances.at(to).read().add(amount);
        storage.public_balances.at(to).write(to_balance);
    }
    // docs:end:transfer_public

    // docs:start:burn_public
    #[aztec(public)]
    fn burn_public(from: AztecAddress, amount: Field, nonce: Field) {
        // docs:start:assert_current_call_valid_authwit_public
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }
        // docs:end:assert_current_call_valid_authwit_public

        let amount = U128::from_integer(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);
        storage.public_balances.at(from).write(from_balance);

        let new_supply = storage.total_supply.read().sub(amount);
        storage.total_supply.write(new_supply);
    }
    // docs:end:burn_public

    // docs:start:redeem_shield
    #[aztec(private)]
    fn redeem_shield(to: AztecAddress, amount: Field, secret: Field) {
        let pending_shields = storage.pending_shields;
        let secret_hash = compute_secret_hash(secret);
        // Get 1 note (set_limit(1)) which has amount stored in field with index 0 (select(0, amount)) and secret_hash
        // stored in field with index 1 (select(1, secret_hash)).
        let mut options = NoteGetterOptions::new();
        options = options.select(TransparentNote::properties().amount, amount, Option::none()).select(
            TransparentNote::properties().secret_hash,
            secret_hash,
            Option::none()
        ).set_limit(1);
        let notes = pending_shields.get_notes(options);
        let note = notes.get_unchecked(0);
        // Remove the note from the pending shields set
        pending_shields.remove(note);

        // Add the token note to user's balances set
        // Note: Using context.msg_sender() as a sender below makes this incompatible with escrows because we send
        // outgoing logs to that address and to send outgoing logs you need to get a hold of ovsk_m.
        let from = context.msg_sender();
        storage.balances.add(to, U128::from_integer(amount)).emit(encode_and_encrypt_note(&mut context, from, to));
    }
    // docs:end:redeem_shield

    // docs:start:unshield
    #[aztec(private)]
    fn unshield(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.sub(from, U128::from_integer(amount)).emit(encode_and_encrypt_note(&mut context, from, from));

        TokenWithRefunds::at(context.this_address())._increase_public_balance(to, amount).enqueue(&mut context);
    }
    // docs:end:unshield

    // docs:start:transfer
    #[aztec(private)]
    fn transfer(to: AztecAddress, amount: Field) {
        let from = context.msg_sender();

        // By fetching the keys here, we can avoid doing an extra read from the storage, since from_ovpk would
        // be needed twice.
        let header = context.get_header();
        let from_ovpk = header.get_ovpk_m(&mut context, from);
        let from_ivpk = header.get_ivpk_m(&mut context, from);
        let to_ivpk = header.get_ivpk_m(&mut context, to);

        let amount = U128::from_integer(amount);
        storage.balances.sub(from, amount).emit(encode_and_encrypt_note_with_keys_unconstrained(&mut context, from_ovpk, from_ivpk, from));
        storage.balances.add(to, amount).emit(encode_and_encrypt_note_with_keys_unconstrained(&mut context, from_ovpk, to_ivpk, to));

        Transfer2 { from, to, amount: amount.to_field() }.emit(encode_and_encrypt_event_with_keys_unconstrained(&mut context, from_ovpk, to_ivpk, to));
    }
    // docs:end:transfer

    /**
     * Cancel a private authentication witness.
     * @param inner_hash The inner hash of the authwit to cancel.
     */
    // docs:start:cancel_authwit
    #[aztec(private)]
    fn cancel_authwit(inner_hash: Field) {
        let on_behalf_of = context.msg_sender();
        let nullifier = compute_authwit_nullifier(on_behalf_of, inner_hash);
        context.push_nullifier(nullifier);
    }
    // docs:end:cancel_authwit

    // docs:start:transfer_from
    #[aztec(private)]
    fn transfer_from(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        // docs:start:assert_current_call_valid_authwit
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }
        // docs:end:assert_current_call_valid_authwit

        // By fetching the keys here, we can avoid doing an extra read from the storage, since from_ovpk would
        // be needed twice.
        let header = context.get_header();
        let from_ovpk = header.get_ovpk_m(&mut context, from);
        let from_ivpk = header.get_ivpk_m(&mut context, from);
        let to_ivpk = header.get_ivpk_m(&mut context, to);

        let amount = U128::from_integer(amount);
        // docs:start:increase_private_balance
        // docs:start:encrypted
        storage.balances.sub(from, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, from_ivpk, from));
        // docs:end:encrypted
        // docs:end:increase_private_balance
        storage.balances.add(to, amount).emit(encode_and_encrypt_note_with_keys(&mut context, from_ovpk, to_ivpk, to));
    }
    // docs:end:transfer_from

    // docs:start:burn
    #[aztec(private)]
    fn burn(from: AztecAddress, amount: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.sub(from, U128::from_integer(amount)).emit(encode_and_encrypt_note(&mut context, from, from));

        TokenWithRefunds::at(context.this_address())._reduce_total_supply(amount).enqueue(&mut context);
    }
    // docs:end:burn

    /// Internal ///

    // docs:start:increase_public_balance
    #[aztec(public)]
    #[aztec(internal)]
    fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.public_balances.at(to).read().add(U128::from_integer(amount));
        storage.public_balances.at(to).write(new_balance);
    }
    // docs:end:increase_public_balance

    // docs:start:reduce_total_supply
    #[aztec(public)]
    #[aztec(internal)]
    fn _reduce_total_supply(amount: Field) {
        // Only to be called from burn.
        let new_supply = storage.total_supply.read().sub(U128::from_integer(amount));
        storage.total_supply.write(new_supply);
    }
    // docs:end:reduce_total_supply

    /// Unconstrained ///

    // docs:start:balance_of_private
    unconstrained fn balance_of_private(owner: AztecAddress) -> pub Field {
        storage.balances.balance_of(owner).to_field()
    }
    // docs:end:balance_of_private

    // REFUNDS SPECIFIC FUNCTIONALITY FOLLOWS
    use dep::aztec::{
        note::utils::compute_slotted_note_hiding_point_raw, prelude::FunctionSelector,
        protocol_types::{storage::map::derive_storage_slot_in_map, point::Point}
    };

    #[aztec(private)]
    fn setup_refund(
        fee_payer: AztecAddress, // Address of the entity which will receive the fee note.
        user: AztecAddress, // A user for which we are setting up the fee refund.
        funded_amount: Field, // The amount the user funded the fee payer with (represents fee limit).
        user_randomness: Field, // A randomness to mix in with the generated refund note for the sponsored user.
        fee_payer_randomness: Field // A randomness to mix in with the generated fee note for the fee payer.
    ) {
        // 1. This function is called by fee paying contract (fee_payer) when setting up a refund so we need to support
        // the authwit flow here and check that the user really permitted fee_payer to set up a refund on their behalf.
        assert_current_call_valid_authwit(&mut context, user);

        // 2. Get all the relevant keys
        let header = context.get_header();
        let fee_payer_npk_m_hash = header.get_npk_m_hash(&mut context, fee_payer);
        let user_npk_m_hash = header.get_npk_m_hash(&mut context, user);
        let user_ovpk = header.get_ovpk_m(&mut context, user);
        let user_ivpk = header.get_ivpk_m(&mut context, user);

        // 3. Deduct the funded amount from the user's balance - this is a maximum fee a user is willing to pay
        // (called fee limit in aztec spec). The difference between fee limit and the actual tx fee will be refunded 
        // to the user in the `complete_refund(...)` function.
        storage.balances.sub(user, U128::from_integer(funded_amount)).emit(encode_and_encrypt_note_with_keys(&mut context, user_ovpk, user_ivpk, user));

        // 4. We generate the refund points.
        let (fee_payer_point, user_point) = TokenNote::generate_refund_points(
            fee_payer_npk_m_hash,
            user_npk_m_hash,
            user_randomness,
            fee_payer_randomness
        );

        // 5. Now we "manually" compute the slots and the slotted note hiding points
        let fee_payer_balances_slot = derive_storage_slot_in_map(TokenWithRefunds::storage().balances.slot, fee_payer);
        let user_balances_slot = derive_storage_slot_in_map(TokenWithRefunds::storage().balances.slot, user);

        let slotted_fee_payer_point = compute_slotted_note_hiding_point_raw(fee_payer_balances_slot, fee_payer_point);
        let slotted_user_point = compute_slotted_note_hiding_point_raw(user_balances_slot, user_point);

        // 6. Set the public teardown function to `complete_refund(...)`. Public teardown is the only time when a public
        // function has access to the final transaction fee, which is needed to compute the actual refund amount.
        context.set_public_teardown_function(
            context.this_address(),
            FunctionSelector::from_signature("complete_refund((Field,Field,bool),(Field,Field,bool),Field)"),
            [
            slotted_fee_payer_point.x, slotted_fee_payer_point.y, slotted_fee_payer_point.is_infinite as Field, slotted_user_point.x, slotted_user_point.y, slotted_user_point.is_infinite as Field, funded_amount
        ]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn complete_refund(fee_payer_point: Point, user_point: Point, funded_amount: Field) {
        // 1. We get the final note hashes by calling a `complete_refund` function on the note.
        // We use 1:1 exchange rate between fee juice and token so just passing transaction fee and funded amount
        // to `complete_refund(...)` function is enough.
        let (fee_payer_note_hash, user_note_hash) = TokenNote::complete_refund(
            fee_payer_point,
            user_point,
            funded_amount,
            context.transaction_fee()
        );

        // 2. At last we emit the note hashes.
        context.push_note_hash(fee_payer_note_hash);
        context.push_note_hash(user_note_hash);
        // --> Once the tx is settled user and fee recipient can add the notes to their pixies.
    }
    // END OF REFUNDS SPECIFIC FUNCTIONALITY
}
// docs:end:token_all