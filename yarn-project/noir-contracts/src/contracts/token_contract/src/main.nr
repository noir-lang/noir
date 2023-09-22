// docs:start:token_all
// docs:start:imports
mod types;
mod util;

// Minimal token implementation that supports `AuthWit` accounts.
// The auth message follows a similar pattern to the cross-chain message and includes a designated caller.
// The designated caller is ALWAYS used here, and not based on a flag as cross-chain.
// message hash = H([caller, contract, selector, ...args])
// To be read as `caller` calls function at `contract` defined by `selector` with `args`
// Including a nonce in the message hash ensures that the message can only be used once.

contract Token {
    // Libs
    use dep::std::option::Option;

    use dep::safe_math::SafeU120;

    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNoteMethods, ValueNote},
    };

    use dep::aztec::{
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        context::{PrivateContext, PublicContext, Context},
        state_vars::{map::Map, public_state::PublicState, set::Set},
        types::type_serialization::field_serialization::{
            FieldSerializationMethods, FIELD_SERIALIZED_LEN,
        },
        types::address::{AztecAddress},
        oracle::compute_selector::compute_selector,
        auth::{assert_valid_message_for, assert_valid_public_message_for}
    };

    use crate::types::{TransparentNote, TransparentNoteMethods, TRANSPARENT_NOTE_LEN};
    use crate::util::{compute_message_hash};
    // docs:end::imports

    // docs:start:storage_struct
    struct Storage {
        admin: PublicState<Field, FIELD_SERIALIZED_LEN>,
        minters: Map<PublicState<Field, FIELD_SERIALIZED_LEN>>,
        balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
        total_supply: PublicState<Field, FIELD_SERIALIZED_LEN>,
        pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,
        public_balances: Map<PublicState<Field, FIELD_SERIALIZED_LEN>>,
    }
    // docs:end:storage_struct

    // docs:start:storage_init
    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                admin: PublicState::new(
                    context,
                    1,
                    FieldSerializationMethods,
                ),
                minters: Map::new(
                    context,
                    2,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerializationMethods,
                        )
                    },
                ),
                balances: Map::new(
                    context,
                    3,
                    |context, slot| {
                        Set::new(context, slot, ValueNoteMethods)
                    },
                ),
                total_supply: PublicState::new(
                    context,
                    4,
                    FieldSerializationMethods,
                ),
                pending_shields: Set::new(context, 5, TransparentNoteMethods),
                public_balances: Map::new(
                    context,
                    6,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerializationMethods,
                        )
                    },
                ),
            }
        }
    }
    // docs:end:storage_init

    // docs:start:constructor
    #[aztec(private)]
    fn constructor() {
        // Currently not possible to execute public calls from constructor as code not yet available to sequencer.
        // let selector = compute_selector("_initialize((Field))");
        // let _callStackItem = context.call_public_function(context.this_address(), selector, [context.msg_sender()]);
    }
    // docs:end:constructor

    // docs:start:set_admin
    #[aztec(public)]
    fn set_admin(
        new_admin: AztecAddress,
    ) {
        let storage = Storage::init(Context::public(&mut context));
        assert(storage.admin.read() == context.msg_sender(), "caller is not admin");
        storage.admin.write(new_admin.address);
    }
    // docs:end:set_admin

    // docs:start:set_minter
    #[aztec(public)]
    fn set_minter(
        minter: AztecAddress,
        approve: bool,
    ) {
        let storage = Storage::init(Context::public(&mut context));
        assert(storage.admin.read() == context.msg_sender(), "caller is not admin");
        storage.minters.at(minter.address).write(approve as Field);
    }
    // docs:end:set_minter

    // docs:start:mint_public
    #[aztec(public)]
    fn mint_public(
        to: AztecAddress,
        amount: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        assert(storage.minters.at(context.msg_sender()).read() == 1, "caller is not minter");
        let amount = SafeU120::new(amount);
        let new_balance = SafeU120::new(storage.public_balances.at(to.address).read()).add(amount);
        let supply = SafeU120::new(storage.total_supply.read()).add(amount);

        storage.public_balances.at(to.address).write(new_balance.value as Field);
        storage.total_supply.write(supply.value as Field);
        1
    }
    // docs:end:mint_public

    // docs:start:mint_private
    #[aztec(public)]
    fn mint_private(
        amount: Field,
        secret_hash: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        assert(storage.minters.at(context.msg_sender()).read() == 1, "caller is not minter");
        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount, secret_hash);
        let supply = SafeU120::new(storage.total_supply.read()).add(SafeU120::new(amount));

        storage.total_supply.write(supply.value as Field);
        pending_shields.insert_from_public(&mut note);
        1
    }
    // docs:end:mint_private

    // docs:start:shield
    #[aztec(public)]
    fn shield(
        from: AztecAddress,
        amount: Field,
        secret_hash: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));

        if (from.address != context.msg_sender()) {
            // The redeem is only spendable once, so we need to ensure that you cannot insert multiple shields from the same message.
            let selector = compute_selector("shield((Field),Field,Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, amount, secret_hash, nonce]);
            assert_valid_public_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = SafeU120::new(storage.public_balances.at(from.address).read()).sub(amount);

        let pending_shields = storage.pending_shields;
        let mut note = TransparentNote::new(amount.value as Field, secret_hash);

        storage.public_balances.at(from.address).write(from_balance.value as Field);
        pending_shields.insert_from_public(&mut note);
        1
    }
    // docs:end:shield

    // docs:start:transfer_public
    #[aztec(public)]
    fn transfer_public(
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));

        if (from.address != context.msg_sender()) {
            let selector = compute_selector("transfer_public((Field),(Field),Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, to.address, amount, nonce]);
            assert_valid_public_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = SafeU120::new(storage.public_balances.at(from.address).read()).sub(amount);
        storage.public_balances.at(from.address).write(from_balance.value as Field);

        let to_balance = SafeU120::new(storage.public_balances.at(to.address).read()).add(amount);
        storage.public_balances.at(to.address).write(to_balance.value as Field);

        1
    }
    // docs:end:transfer_public

    // docs:start:burn_public
    #[aztec(public)]
    fn burn_public(
        from: AztecAddress,
        amount: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));

        if (from.address != context.msg_sender()) {
            let selector = compute_selector("burn_public((Field),Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, amount, nonce]);
            assert_valid_public_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let amount = SafeU120::new(amount);
        let from_balance = SafeU120::new(storage.public_balances.at(from.address).read()).sub(amount);
        storage.public_balances.at(from.address).write(from_balance.value as Field);

        let new_supply = SafeU120::new(storage.total_supply.read()).sub(amount);
        storage.total_supply.write(new_supply.value as Field);

        1
    }
    // docs:end:burn_public

    // docs:start:redeem_shield
    #[aztec(private)]
    fn redeem_shield(
        to: AztecAddress,
        amount: Field,
        secret: Field,
    ) -> Field {
        let storage = Storage::init(Context::private(&mut context));
        let pending_shields = storage.pending_shields;
        let balance = storage.balances.at(to.address);
        let mut public_note = TransparentNote::new_from_secret(amount, secret);
        
        pending_shields.assert_contains_and_remove_publicly_created(&mut public_note);
        increment(balance, amount, to.address);

        1
    }
    // docs:end:redeem_shield

    // docs:start:unshield
    #[aztec(private)]
    fn unshield(
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::private(&mut context));

        if (from.address != context.msg_sender()) {
            let selector = compute_selector("unshield((Field),(Field),Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, to.address, amount, nonce]);
            assert_valid_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let from_balance = storage.balances.at(from.address);
        decrement(from_balance, amount, from.address);

        let selector = compute_selector("_increase_public_balance((Field),Field)");
        let _void = context.call_public_function(context.this_address(), selector, [to.address, amount]);

        1
    }
    // docs:end:unshield

    // docs:start:transfer
    #[aztec(private)]
    fn transfer(
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::private(&mut context));

        if (from.address != context.msg_sender()) {
            let selector = compute_selector("transfer((Field),(Field),Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, to.address, amount, nonce]);
            assert_valid_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let from_balance = storage.balances.at(from.address);
        let to_balance = storage.balances.at(to.address);

        decrement(from_balance, amount, from.address);
        increment(to_balance, amount, to.address);

        1
    }
    // docs:end:transfer
    
    // docs:start:burn
    #[aztec(private)]
    fn burn(
        from: AztecAddress,
        amount: Field,
        nonce: Field,
    ) -> Field {
        let storage = Storage::init(Context::private(&mut context));

        if (from.address != context.msg_sender()) {
            let selector = compute_selector("burn((Field),Field,Field)");
            let message_field = compute_message_hash([context.msg_sender(), context.this_address(), selector, from.address, amount, nonce]);
            assert_valid_message_for(&mut context, from.address, message_field);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let from_balance = storage.balances.at(from.address);

        decrement(from_balance, amount, from.address);

        let selector = compute_selector("_reduce_total_supply(Field)");
        let _void = context.call_public_function(context.this_address(), selector, [amount]);

        1
    }
    // docs:end:burn

    /// SHOULD BE Internal ///

    // docs:start:initialize
    // We cannot do this from the constructor currently 
    // Since this should be internal, for now, we ignore the safety checks of it, as they are 
    // enforced by it being internal and only called from the constructor.
    #[aztec(public)]
    fn _initialize(
        new_admin: AztecAddress,
    ) {
        let storage = Storage::init(Context::public(&mut context));
        storage.admin.write(new_admin.address);
        storage.minters.at(new_admin.address).write(1);
    }
    // docs:end:initialize

    /// Internal ///

    // docs:start:increase_public_balance
    #[aztec(public)]
    internal fn _increase_public_balance(
        to: AztecAddress,
        amount: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));
        let new_balance = SafeU120::new(storage.public_balances.at(to.address).read()).add(SafeU120::new(amount));
        storage.public_balances.at(to.address).write(new_balance.value as Field);
    }
    // docs:end:increase_public_balance

    // docs:start:reduce_total_supply
    #[aztec(public)]
    internal fn _reduce_total_supply(
        amount: Field,
    ) {
        // Only to be called from burn.
        let storage = Storage::init(Context::public(&mut context));
        let new_supply = SafeU120::new(storage.total_supply.read()).sub(SafeU120::new(amount));
        storage.total_supply.write(new_supply.value as Field);
    }
    // docs:end:reduce_total_supply

    /// Unconstrained /// 

    // docs:start:admin
    unconstrained fn admin() -> Field {
        let storage = Storage::init(Context::none());
        storage.admin.read()
    }
    // docs:end:admin

    // docs:start:is_minter
    unconstrained fn is_minter(
        minter: AztecAddress,
    ) -> bool {
        let storage = Storage::init(Context::none());
        storage.minters.at(minter.address).read() as bool
    }
    // docs:end:is_minter

    // docs:start:total_supply
    unconstrained fn total_supply() -> Field {
        let storage = Storage::init(Context::none());
        storage.total_supply.read()
    }
    // docs:end:total_supply

    // docs:start:balance_of_private
    unconstrained fn balance_of_private(
        owner: AztecAddress,
    ) -> Field {
        let storage = Storage::init(Context::none());
        let owner_balance = storage.balances.at(owner.address);

        balance_utils::get_balance(owner_balance)
    }
    // docs:end:balance_of_private

    // docs:start:balance_of_public
    unconstrained fn balance_of_public(
        owner: AztecAddress,
    ) -> Field {
        let storage = Storage::init(Context::none());
        storage.public_balances.at(owner.address).read()
    }
    // docs:end:balance_of_public

    // Below this point is the stuff of nightmares.
    // This should ideally not be required. What do we do if vastly different types of preimages?

    // docs:start:compute_note_hash_and_nullifier
    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 5) {
            note_utils::compute_note_hash_and_nullifier(TransparentNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
        }
    }
    // docs:end:compute_note_hash_and_nullifier

}
// docs:end:token_all
