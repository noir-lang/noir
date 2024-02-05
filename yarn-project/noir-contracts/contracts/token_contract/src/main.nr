// docs:start:token_all
// docs:start:imports
mod types;

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
    use dep::compressed_string::FieldCompressedString;

    use dep::aztec::{
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        context::{PrivateContext, PublicContext, Context},
        hash::{compute_secret_hash},
        state_vars::{map::Map, public_state::PublicState, stable_public_state::StablePublicState,  set::Set},
        protocol_types::{
            abis::function_selector::FunctionSelector,
            address::AztecAddress
        }
    };

    // docs:start:import_authwit
    use dep::authwit::{
        auth::{
            assert_current_call_valid_authwit,
            assert_current_call_valid_authwit_public,
        },
    };
    // docs:end:import_authwit

    use crate::types::{
        transparent_note::TransparentNote,
        token_note::{TokenNote, TOKEN_NOTE_LEN},
        balances_map::BalancesMap
    };
    // docs:end::imports

    // docs:start:storage_struct
    struct Storage {
        // docs:start:storage_admin
        admin: PublicState<AztecAddress>,
        // docs:end:storage_admin
        // docs:start:storage_minters
        minters: Map<AztecAddress, PublicState<bool>>, 
        // docs:end:storage_minters
        // docs:start:storage_balances
        balances: BalancesMap<TokenNote>,
        // docs:end:storage_balances
        total_supply: PublicState<SafeU120>, 
        // docs:start:storage_pending_shields
        pending_shields: Set<TransparentNote>, 
        // docs:end:storage_pending_shields
        public_balances: Map<AztecAddress, PublicState<SafeU120>>,
        symbol: StablePublicState<FieldCompressedString>,
        name: StablePublicState<FieldCompressedString>,
        // docs:start:storage_decimals
        decimals: StablePublicState<u8>,
        // docs:end:storage_decimals
    }
    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(private)]
    fn constructor(admin: AztecAddress, name: str<31>, symbol: str<31>, decimals: u8) {
        let selector = FunctionSelector::from_signature("_initialize((Field),(Field),(Field),u8)");
        let name_s = FieldCompressedString::from_string(name);
        let symbol_s = FieldCompressedString::from_string(symbol);
        context.call_public_function(
            context.this_address(),
            selector,
            [admin.to_field(), name_s.serialize()[0], symbol_s.serialize()[0], decimals as Field]
        );
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
        // docs:start:read_decimals_public
        storage.decimals.read_public()
        // docs:end:read_decimals_public
    }

    #[aztec(private)]
    fn private_get_decimals() -> pub u8 {
        // docs:start:read_decimals_private
        storage.decimals.read_private()
        // docs:end:read_decimals_private
    }

    unconstrained fn un_get_decimals() -> pub u8 {
        storage.decimals.read_public()
    }

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
        let amount = SafeU120::new(amount);
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
        let supply = storage.total_supply.read().add(SafeU120::new(amount));

        storage.total_supply.write(supply);
        // docs:start:insert_from_public
        pending_shields.insert_from_public(&mut note);
        // docs:end:insert_from_public
    }
    // docs:end:mint_private

    // docs:start:shield
    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) {
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
    // docs:end:shield

    // docs:start:transfer_public
    #[aztec(public)]
    fn transfer_public(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
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

        let amount = SafeU120::new(amount);
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
        let options = NoteGetterOptions::new().select(0, amount, Option::none()).select(1, secret_hash, Option::none()).set_limit(1);
        let notes = pending_shields.get_notes(options);
        let note = notes[0].unwrap_unchecked();
        // Remove the note from the pending shields set 
        pending_shields.remove(note);

        // Add the token note to user's balances set
        storage.balances.add(to, SafeU120::new(amount));
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

        storage.balances.sub(from, SafeU120::new(amount));

        let selector = FunctionSelector::from_signature("_increase_public_balance((Field),Field)");
        let _void = context.call_public_function(context.this_address(), selector, [to.to_field(), amount]);
    }
    // docs:end:unshield

    // docs:start:transfer
    #[aztec(private)]
    fn transfer(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
        // docs:start:assert_current_call_valid_authwit
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }
        // docs:end:assert_current_call_valid_authwit

        let amount = SafeU120::new(amount);
        storage.balances.sub(from, amount);
        // docs:start:increase_private_balance
        storage.balances.add(to, amount);
        // docs:end:increase_private_balance
    }
    // docs:end:transfer

    // docs:start:burn
    #[aztec(private)]
    fn burn(from: AztecAddress, amount: Field, nonce: Field) {
        if (!from.eq(context.msg_sender())) {
            assert_current_call_valid_authwit(&mut context, from);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        storage.balances.sub(from, SafeU120::new(amount));

        let selector = FunctionSelector::from_signature("_reduce_total_supply(Field)");
        let _void = context.call_public_function(context.this_address(), selector, [amount]);
    }
    // docs:end:burn

    // docs:start:initialize
    #[aztec(public)]
    internal fn _initialize(
        new_admin: AztecAddress,
        name: FieldCompressedString,
        symbol: FieldCompressedString,
        decimals: u8
    ) {
        assert(!new_admin.is_zero(), "invalid admin");
        storage.admin.write(new_admin);
        storage.minters.at(new_admin).write(true);
        storage.name.initialize(name);
        storage.symbol.initialize(symbol);
        // docs:start:initialize_decimals
        storage.decimals.initialize(decimals);
        // docs:end:initialize_decimals
    }
    // docs:end:initialize

    /// Internal ///

    // docs:start:increase_public_balance
    #[aztec(public)]
    internal fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.public_balances.at(to).read().add(SafeU120::new(amount));
        storage.public_balances.at(to).write(new_balance);
    }
    // docs:end:increase_public_balance

    // docs:start:reduce_total_supply
    #[aztec(public)]
    internal fn _reduce_total_supply(amount: Field) {
        // Only to be called from burn.
        let new_supply = storage.total_supply.read().sub(SafeU120::new(amount));
        storage.total_supply.write(new_supply);
    }
    // docs:end:reduce_total_supply

    /// Unconstrained ///

    // docs:start:admin
    unconstrained fn admin() -> pub Field {
        storage.admin.read().to_field()
    }
    // docs:end:admin

    // docs:start:is_minter
    unconstrained fn is_minter(minter: AztecAddress) -> pub bool {
        storage.minters.at(minter).read()
    }
    // docs:end:is_minter

    // docs:start:total_supply
    unconstrained fn total_supply() -> pub u120 {
        storage.total_supply.read().value
    }
    // docs:end:total_supply

    // docs:start:balance_of_private
    unconstrained fn balance_of_private(owner: AztecAddress) -> pub u120 {
        storage.balances.balance_of(owner).value
    }
    // docs:end:balance_of_private

    // docs:start:balance_of_public
    unconstrained fn balance_of_public(owner: AztecAddress) -> pub u120 {
        storage.public_balances.at(owner).read().value
    }
    // docs:end:balance_of_public

    // Below this point is the stuff of nightmares.
    // This should ideally not be required. What do we do if vastly different types of serialized_notes?

    // docs:start:compute_note_hash_and_nullifier
    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; TOKEN_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        if (storage_slot == storage.pending_shields.get_storage_slot()) {
            note_utils::compute_note_hash_and_nullifier(
                TransparentNote::deserialize_content,
                note_header,
                serialized_note
            )
        } else {
            note_utils::compute_note_hash_and_nullifier(TokenNote::deserialize_content, note_header, serialized_note)
        }
    }
    // docs:end:compute_note_hash_and_nullifier
}
// docs:end:token_all
