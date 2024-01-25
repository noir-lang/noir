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
    use dep::std::option::Option;

    use dep::safe_math::SafeU120;
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::AztecAddress,
    };
    use dep::aztec::{
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        context::{PrivateContext, PublicContext, Context},
        hash::{compute_secret_hash},
        state_vars::{map::Map, public_state::PublicState, set::Set, immutable_singleton::ImmutableSingleton},
        types::type_serialization::{
            field_serialization::{FieldSerializationMethods, FIELD_SERIALIZED_LEN},
            bool_serialization::{BoolSerializationMethods, BOOL_SERIALIZED_LEN},
            address_serialization::{AddressSerializationMethods, AZTEC_ADDRESS_SERIALIZED_LEN},
        },
    };

    use dep::field_note::field_note::{FieldNote, FieldNoteMethods, FIELD_NOTE_LEN};

    use dep::authwit::{
        auth::{
            assert_current_call_valid_authwit,
            assert_current_call_valid_authwit_public,
        },
    };

    use crate::types::{
        transparent_note::{TransparentNote, TransparentNoteMethods, TRANSPARENT_NOTE_LEN},
        token_note::{TokenNote, TokenNoteMethods, TOKEN_NOTE_LEN},
        balances_map::{BalancesMap},
        safe_u120_serialization::{SafeU120SerializationMethods, SAFE_U120_SERIALIZED_LEN},
        roles::UserFlags,
    };
    // docs:start:interface
    use crate::interfaces::SlowMap;
    // docs:end:interface

    struct Storage {
        admin: PublicState<AztecAddress, AZTEC_ADDRESS_SERIALIZED_LEN>,
        balances: BalancesMap,
        total_supply: PublicState<SafeU120, SAFE_U120_SERIALIZED_LEN>,
        pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,
        public_balances: Map<AztecAddress, PublicState<SafeU120, SAFE_U120_SERIALIZED_LEN>>,
        slow_update: ImmutableSingleton<FieldNote, FIELD_NOTE_LEN>,
        public_slow_update: PublicState<AztecAddress, AZTEC_ADDRESS_SERIALIZED_LEN>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                admin: PublicState::new(
                    context,
                    1,
                    AddressSerializationMethods,
                ),
                balances: BalancesMap::new(context, 3),
                total_supply: PublicState::new(
                    context,
                    4,
                    SafeU120SerializationMethods,
                ),
                pending_shields: Set::new(context, 5, TransparentNoteMethods),
                public_balances: Map::new(
                    context,
                    6,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            SafeU120SerializationMethods,
                        )
                    },
                ),
                // Below is an abomination to have same value in private and public (immutable in solidity).
                // docs:start:slow_updates_storage
                slow_update: ImmutableSingleton::new(context, 7, FieldNoteMethods),
                public_slow_update: PublicState::new(
                    context,
                    8,
                    AddressSerializationMethods,
                ),
                // docs:end:slow_updates_storage

            }
        }
    }
    // docs:start:constructor
    #[aztec(private)]
    fn constructor(admin: AztecAddress, slow_updates_contract: AztecAddress) {
        let mut slow_note = FieldNote::new(slow_updates_contract.to_field());
        storage.slow_update.initialize(&mut slow_note, false);
        // docs:end:constructor
        let selector = FunctionSelector::from_signature("_initialize((Field),(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [admin.to_field(), slow_updates_contract.to_field()]
        );
        // We cannot do the following atm
        // let roles = UserFlags{is_admin: true, is_minter: false, is_blacklisted: false}.get_value() as Field;
        // SlowMap::at(slow_updates_contract).update_at_private(&mut context, admin.to_field(), roles);
    }

    ////////
    // Looking ugly because we cannot do constructor -> private calls

    #[aztec(private)]
    fn init_slow_tree(user: AztecAddress) {
        let roles = UserFlags { is_admin: true, is_minter: false, is_blacklisted: false }.get_value() as Field;
        // docs:start:get_and_update_private
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        slow.update_at_private(&mut context, user.to_field(), roles);
        // docs:end:get_and_update_private
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("_init_slow_tree((Field))"),
            [context.msg_sender().to_field()]
        );
    }

    #[aztec(public)]
    internal fn _init_slow_tree(caller: AztecAddress) {
        assert(storage.admin.read().eq(caller), "caller is not admin");
    }

    ///////
    #[aztec(public)]
    internal fn _initialize(new_admin: AztecAddress, slow_updates_contract: AztecAddress) {
        assert(!new_admin.is_zero(), "invalid admin");
        storage.admin.write(new_admin);
        // docs:start:write_slow_update_public
        storage.public_slow_update.write(slow_updates_contract);
        // docs:end:write_slow_update_public
        // docs:start:slowmap_initialize
        SlowMap::at(slow_updates_contract).initialize(context);
        // docs:end:slowmap_initialize
    }

    #[aztec(private)]
    fn update_roles(user: AztecAddress, roles: Field) {
        // docs:start:slowmap_at
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        // docs:end:slowmap_at
        let caller_roles = UserFlags::new(slow.read_at(&mut context, context.msg_sender().to_field()) as u120);
        assert(caller_roles.is_admin, "caller is not admin");

        slow.update_at_private(&mut context, user.to_field(), roles);
    }

    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        // docs:start:get_public
        let slow = SlowMap::at(storage.public_slow_update.read());
        // docs:end:get_public
        // docs:start:read_at_pub
        let to_roles = UserFlags::new(slow.read_at_pub(context, to.to_field()) as u120);
        // docs:end:read_at_pub
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        let caller_roles = UserFlags::new(slow.read_at_pub(context, context.msg_sender().to_field()) as u120);
        assert(caller_roles.is_minter, "caller is not minter");

        let amount = SafeU120::new(amount);
        let new_balance = storage.public_balances.at(to).read().add(amount);
        let supply = storage.total_supply.read().add(amount);

        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(supply);
    }

    #[aztec(public)]
    fn mint_private(amount: Field, secret_hash: Field) {
        let slow = SlowMap::at(storage.public_slow_update.read());
        let caller_roles = UserFlags::new(slow.read_at_pub(context, context.msg_sender().to_field()) as u120);
        assert(caller_roles.is_minter, "caller is not minter");

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount, secret_hash);
        let supply = storage.total_supply.read().add(SafeU120::new(amount));

        storage.total_supply.write(supply);
        pending_shields.insert_from_public(&mut note);
    }

    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) {
        let slow = SlowMap::at(storage.public_slow_update.read());
        let from_roles = UserFlags::new(slow.read_at_pub(context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

        if (!from.eq(context.msg_sender())) {
            // The redeem is only spendable once, so we need to ensure that you cannot insert multiple shields from the same message.
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount.value as Field, secret_hash);

        storage.public_balances.at(from).write(from_balance);
        pending_shields.insert_from_public(&mut note);
    }

    #[aztec(public)]
    fn transfer_public(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.public_slow_update.read());
        let from_roles = UserFlags::new(slow.read_at_pub(context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(slow.read_at_pub(context, to.to_field()) as u120);
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);
        storage.public_balances.at(from).write(from_balance);

        let to_balance = storage.public_balances.at(to).read().add(amount);
        storage.public_balances.at(to).write(to_balance);
    }

    #[aztec(public)]
    fn burn_public(from: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(storage.public_slow_update.read());
        let from_roles = UserFlags::new(slow.read_at_pub(context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = storage.public_balances.at(from).read().sub(amount);
        storage.public_balances.at(from).write(from_balance);

        let new_supply = storage.total_supply.read().sub(amount);
        storage.total_supply.write(new_supply);
    }

    #[aztec(private)]
    fn redeem_shield(to: AztecAddress, amount: Field, secret: Field) {
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        // docs:start:slowmap_read_at
        let to_roles = UserFlags::new(slow.read_at(&mut context, to.to_field()) as u120);
        // docs:end:slowmap_read_at
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        let pending_shields = storage.pending_shields;
        let secret_hash = compute_secret_hash(secret);
        // Get 1 note (set_limit(1)) which has amount stored in field with index 0 (select(0, amount)) and secret_hash
        // stored in field with index 1 (select(1, secret_hash)).
        let options = NoteGetterOptions::new().select(0, amount, Option::none()).select(1, secret_hash, Option::none()).set_limit(1);
        let notes = pending_shields.get_notes(options);
        let note = notes[0].unwrap_unchecked();
        // Remove the note from the pending shields set
        pending_shields.remove(note);

        // Add the token note to user's balances set
        storage.balances.at(to).add(SafeU120::new(amount));
    }

    #[aztec(private)]
    fn unshield(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        let from_roles = UserFlags::new(slow.read_at(&mut context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(slow.read_at(&mut context, to.to_field()) as u120);
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.at(from).sub(SafeU120::new(amount));

        let selector = FunctionSelector::from_signature("_increase_public_balance((Field),Field)");
        context.call_public_function(context.this_address(), selector, [to.to_field(), amount]);
    }

    // docs:start:transfer_private
    #[aztec(private)]
    fn transfer(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        let from_roles = UserFlags::new(slow.read_at(&mut context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");
        let to_roles = UserFlags::new(slow.read_at(&mut context, to.to_field()) as u120);
        assert(!to_roles.is_blacklisted, "Blacklisted: Recipient");
        // docs:end:transfer_private

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        storage.balances.at(from).sub(amount);
        storage.balances.at(to).add(amount);
    }

    #[aztec(private)]
    fn burn(from: AztecAddress, amount: Field, nonce: Field) {
        let slow = SlowMap::at(AztecAddress::from_field(storage.slow_update.get_note().value));
        let from_roles = UserFlags::new(slow.read_at(&mut context, from.to_field()) as u120);
        assert(!from_roles.is_blacklisted, "Blacklisted: Sender");

        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.at(from).sub(SafeU120::new(amount));

        let selector = FunctionSelector::from_signature("_reduce_total_supply(Field)");
        context.call_public_function(context.this_address(), selector, [amount]);
    }

    /// Internal ///

    #[aztec(public)]
    internal fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.public_balances.at(to).read().add(SafeU120::new(amount));
        storage.public_balances.at(to).write(new_balance);
    }

    #[aztec(public)]
    internal fn _reduce_total_supply(amount: Field) {
        // Only to be called from burn.
        let new_supply = storage.total_supply.read().sub(SafeU120::new(amount));
        storage.total_supply.write(new_supply);
    }

    /// Unconstrained ///

    unconstrained fn total_supply() -> pub u120 {
        storage.total_supply.read().value
    }

    unconstrained fn balance_of_private(owner: AztecAddress) -> pub u120 {
        storage.balances.at(owner).balance_of().value
    }

    unconstrained fn balance_of_public(owner: AztecAddress) -> pub u120 {
        storage.public_balances.at(owner).read().value
    }

    // Below this point is the stuff of nightmares.
    // This should ideally not be required. What do we do if vastly different types of preimages?

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        preimage: [Field; TOKEN_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        if (storage_slot == 5) {
            note_utils::compute_note_hash_and_nullifier(TransparentNoteMethods, note_header, preimage)
        } else if (storage_slot == 7) {
            note_utils::compute_note_hash_and_nullifier(FieldNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(TokenNoteMethods, note_header, preimage)
        }
    }
}
