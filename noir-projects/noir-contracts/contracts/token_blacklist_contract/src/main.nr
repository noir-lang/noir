mod types;

// Minimal token implementation that supports `AuthWit` accounts and SharedMutable variables.
// The auth message follows a similar pattern to the cross-chain message and includes a designated caller.
// The designated caller is ALWAYS used here, and not based on a flag as cross-chain.
// message hash = H([caller, contract, selector, ...args])
// To be read as `caller` calls function at `contract` defined by `selector` with `args`
// Including a nonce in the message hash ensures that the message can only be used once.
// The SharedMutables are used for access control related to minters and blacklist.

contract TokenBlacklist {
    // Libs

    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
    use dep::aztec::{
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader},
        hash::compute_secret_hash,
        state_vars::{Map, PublicMutable, PrivateSet, SharedMutable, SharedImmutable}
    };

    use dep::authwit::{auth::{assert_current_call_valid_authwit, assert_current_call_valid_authwit_public}};

    use crate::types::{transparent_note::TransparentNote, token_note::TokenNote, balances_map::BalancesMap, roles::UserFlags};

    // Changing an address' roles has a certain block delay before it goes into effect.
    global CHANGE_ROLES_DELAY_BLOCKS = 5;

    #[aztec(storage)]
    struct Storage {
        balances: BalancesMap<TokenNote>,
        total_supply: PublicMutable<U128>,
        pending_shields: PrivateSet<TransparentNote>,
        public_balances: Map<AztecAddress, PublicMutable<U128>>,
        roles: Map<AztecAddress, SharedMutable<UserFlags, CHANGE_ROLES_DELAY_BLOCKS>>,
    }

    // docs:start:constructor
    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress) {
        let admin_roles = UserFlags { is_admin: true, is_minter: false, is_blacklisted: false };
        storage.roles.at(admin).schedule_value_change(admin_roles);
    }

    #[aztec(public)]
    fn get_roles(user: AztecAddress) -> UserFlags {
        storage.roles.at(user).get_current_value_in_public()
    }

    #[aztec(public)]
    fn update_roles(user: AztecAddress, roles: UserFlags) {
        let caller_roles = storage.roles.at(context.msg_sender()).get_current_value_in_public();
        assert(caller_roles.is_admin, "caller is not admin");

        storage.roles.at(user).schedule_value_change(roles);
    }

    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        let to_roles = storage.roles.at(to).get_current_value_in_public();
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        let caller_roles = storage.roles.at(context.msg_sender()).get_current_value_in_public();
        assert(caller_roles.is_minter, "caller is not minter");

        let amount = U128::from_integer(amount);
        let new_balance = storage.public_balances.at(to).read().add(amount);
        let supply = storage.total_supply.read().add(amount);

        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(supply);
    }

    #[aztec(public)]
    fn mint_private(amount: Field, secret_hash: Field) {
        let caller_roles = storage.roles.at(context.msg_sender()).get_current_value_in_public();
        assert(caller_roles.is_minter, "caller is not minter");

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount, secret_hash);
        let supply = storage.total_supply.read().add(U128::from_integer(amount));

        storage.total_supply.write(supply);
        pending_shields.insert_from_public(&mut note);
    }

    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) {
        let from_roles = storage.roles.at(from).get_current_value_in_public();
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
        let from_roles = storage.roles.at(from).get_current_value_in_public();
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = storage.roles.at(to).get_current_value_in_public();
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
        let from_roles = storage.roles.at(from).get_current_value_in_public();
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
        let to_roles = storage.roles.at(to).get_current_value_in_private();
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

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
        let note = notes[0].unwrap_unchecked();
        // Remove the note from the pending shields set
        pending_shields.remove(note);

        // Add the token note to user's balances set
        storage.balances.add(to, U128::from_integer(amount));
    }

    #[aztec(private)]
    fn unshield(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let from_roles = storage.roles.at(from).get_current_value_in_private();
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = storage.roles.at(to).get_current_value_in_private();
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
        let from_roles = storage.roles.at(from).get_current_value_in_private();
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = storage.roles.at(to).get_current_value_in_private();
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

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
        let from_roles = storage.roles.at(from).get_current_value_in_private();
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
