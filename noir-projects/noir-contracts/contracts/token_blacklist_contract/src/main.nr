mod types;
mod interfaces;

// Minimal token implementation that supports `AuthWit` accounts and the slow update tree.
// The auth message follows a similar pattern to the cross-chain message and includes a designated caller.
// The designated caller is ALWAYS used here, and not based on a flag as cross-chain.
// message hash = H([caller, contract, selector, ...args])
// To be read as `caller` calls function at `contract` defined by `selector` with `args`
// Including a nonce in the message hash ensures that the message can only be used once.
// The slow updates tree are used for access control related to minters and blacklist.

// TODO's: https://github.com/AztecProtocol/aztec-packages/issues/3210
// We are currently unable to do constructor -> private calls

contract TokenBlacklist {
    // Libs

    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
    use dep::aztec::{
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader},
        hash::compute_secret_hash, state_vars::{Map, PublicMutable, PrivateSet, SharedImmutable}
    };

    use dep::field_note::field_note::FieldNote;

    use dep::authwit::{auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public}};

    use crate::types::{transparent_note::TransparentNote, token_note::TokenNote, balances_map::BalancesMap, roles::UserFlags};
    // docs:start:interface
    use crate::interfaces::SlowMap;
    // docs:end:interface

    struct Storage {
        admin: PublicMutable<AztecAddress>,
        balances: BalancesMap<TokenNote>,
        total_supply: PublicMutable<U128>,
        pending_shields: PrivateSet<TransparentNote>,
        public_balances: Map<AztecAddress, PublicMutable<U128>>,
        slow_update: SharedImmutable<AztecAddress>,
    }

    // docs:start:constructor
    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress, slow_updates_contract: AztecAddress) {
        // docs:end:constructor
        assert(!admin.is_zero(), "invalid admin");
        storage.admin.write(admin);
        // docs:start:write_slow_update_public
        storage.slow_update.initialize(slow_updates_contract);
        // docs:end:write_slow_update_public
        // docs:start:slowmap_initialize
        SlowMap::at(slow_updates_contract).initialize(&mut context);
        // docs:end:slowmap_initialize
        // We cannot do the following atm
        // let roles = UserFlags { is_admin: true, is_minter: false, is_blacklisted: false }.get_value().to_field();
        // SlowMap::at(slow_updates_contract).update_at_private(&mut context, admin.to_field(), roles);
    }

    #[aztec(private)]
    fn init_slow_tree(user: AztecAddress) {
        let roles = UserFlags { is_admin: true, is_minter: false, is_blacklisted: false }.get_value().to_field();
        // docs:start:get_and_update_private
        let slow = SlowMap::at(storage.slow_update.read_private());
        slow.update_at_private(&mut context, user.to_field(), roles);
        // docs:end:get_and_update_private
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("_init_slow_tree((Field))"),
            [context.msg_sender().to_field()]
        );
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _init_slow_tree(caller: AztecAddress) {
        assert(storage.admin.read().eq(caller), "caller is not admin");
    }

    #[aztec(private)]
    fn update_roles(user: AztecAddress, roles: Field) {
        // docs:start:slowmap_at
        let slow = SlowMap::at(storage.slow_update.read_private());
        // docs:end:slowmap_at
        let caller_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, context.msg_sender().to_field())));
        assert(caller_roles.is_admin, "caller is not admin");

        slow.update_at_private(&mut context, user.to_field(), roles);
    }

    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        // docs:start:get_public
        let slow = SlowMap::at(storage.slow_update.read_public());
        // docs:end:get_public
        // docs:start:read_at_pub
        let to_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, to.to_field())));
        // docs:end:read_at_pub
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        let caller_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, context.msg_sender().to_field())));
        assert(caller_roles.is_minter, "caller is not minter");

        let amount = U128::from_integer(amount);
        let new_balance = storage.public_balances.at(to).read().add(amount);
        let supply = storage.total_supply.read().add(amount);

        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(supply);
    }

    #[aztec(public)]
    fn mint_private(amount: Field, secret_hash: Field) {
        let slow = SlowMap::at(storage.slow_update.read_public());
        let caller_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, context.msg_sender().to_field())));
        assert(caller_roles.is_minter, "caller is not minter");

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount, secret_hash);
        let supply = storage.total_supply.read().add(U128::from_integer(amount));

        storage.total_supply.write(supply);
        pending_shields.insert_from_public(&mut note);
    }

    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_public());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

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

    #[aztec(public)]
    fn transfer_public(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_public());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, to.to_field())));
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

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

    #[aztec(public)]
    fn burn_public(from: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_public());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = U128::from_integer(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);
        storage.public_balances.at(from).write(from_balance);

        let new_supply = storage.total_supply.read().sub(amount);
        storage.total_supply.write(new_supply);
    }

    #[aztec(private)]
    fn redeem_shield(to: AztecAddress, amount: Field, secret: Field) {
        let slow = SlowMap::at(storage.slow_update.read_private());
        // docs:start:slowmap_read_at
        let to_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, to.to_field())));
        // docs:end:slowmap_read_at
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        let pending_shields = storage.pending_shields;
        let secret_hash = compute_secret_hash(secret);
        // Get 1 note (set_limit(1)) which has amount stored in field with index 0 (select(0, amount)) and secret_hash
        // stored in field with index 1 (select(1, secret_hash)).
        let options = NoteGetterOptions::new().select(TransparentNote::properties().amount, amount, Option::none()).select(
            TransparentNote::properties().secret_hash,
            secret_hash,
            Option::none()
        ).set_limit(1);
        let notes = pending_shields.get_notes(options);
        let note = notes[0].unwrap_unchecked();
        // Remove the note from the pending shields set
        pending_shields.remove(note);

        // Add the token note to user's balances set
        storage.balances.add(to, U128::from_integer(amount));
    }

    #[aztec(private)]
    fn unshield(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_private());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, to.to_field())));
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.sub(from, U128::from_integer(amount));

        let selector = FunctionSelector::from_signature("_increase_public_balance((Field),Field)");
        context.call_public_function(context.this_address(), selector, [to.to_field(), amount]);
    }

    // docs:start:transfer_private
    #[aztec(private)]
    fn transfer(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_private());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, to.to_field())));
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");
        // docs:end:transfer_private

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = U128::from_integer(amount);
        storage.balances.sub(from, amount);
        storage.balances.add(to, amount);
    }

    #[aztec(private)]
    fn burn(from: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.slow_update.read_private());
        let from_roles = UserFlags::new(U128::from_integer(slow.read_at(&mut context, from.to_field())));
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.sub(from, U128::from_integer(amount));

        let selector = FunctionSelector::from_signature("_reduce_total_supply(Field)");
        context.call_public_function(context.this_address(), selector, [amount]);
    }

    /// Internal ///

    #[aztec(public)]
    #[aztec(internal)]
    fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.public_balances.at(to).read().add(U128::from_integer(amount));
        storage.public_balances.at(to).write(new_balance);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _reduce_total_supply(amount: Field) {
        // Only to be called from burn.
        let new_supply = storage.total_supply.read().sub(U128::from_integer(amount));
        storage.total_supply.write(new_supply);
    }

    /// Unconstrained ///

    unconstrained fn total_supply() -> pub Field {
        storage.total_supply.read().to_field()
    }

    unconstrained fn balance_of_private(owner: AztecAddress) -> pub Field {
        storage.balances.balance_of(owner).to_field()
    }

    unconstrained fn balance_of_public(owner: AztecAddress) -> pub Field {
        storage.public_balances.at(owner).read().to_field()
    }
}
