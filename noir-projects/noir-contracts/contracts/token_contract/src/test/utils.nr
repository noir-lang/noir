use dep::aztec::{
    hash::compute_secret_hash, prelude::AztecAddress,
    test::helpers::{cheatcodes, test_environment::TestEnvironment},
    protocol_types::storage::map::derive_storage_slot_in_map,
    note::{note_getter::{MAX_NOTES_PER_PAGE, view_notes}, note_viewer_options::NoteViewerOptions},
    oracle::{execution::{get_block_number, get_contract_address}, unsafe_rand::unsafe_rand, storage::storage_read}
};

use crate::{types::{token_note::TokenNote, transparent_note::TransparentNote}, Token};

pub fn setup(with_account_contracts: bool) -> (&mut TestEnvironment, AztecAddress, AztecAddress, AztecAddress) {
    // Setup env, generate keys
    let mut env = TestEnvironment::new();
    let (owner, recipient) = if with_account_contracts {
        let owner = env.create_account_contract(1);
        let recipient = env.create_account_contract(2);
        // Deploy canonical auth registry
        let _auth_registry = env.deploy("./@auth_registry_contract", "AuthRegistry").without_initializer();
        (owner, recipient)
    } else {
        let owner = env.create_account();
        let recipient = env.create_account();
        (owner, recipient)
    };

    // Start the test in the account contract address
    env.impersonate(owner);

    // Deploy token contract
    let initializer_call_interface = Token::interface().constructor(
        owner,
        "TestToken0000000000000000000000",
        "TT00000000000000000000000000000",
        18
    );
    let token_contract = env.deploy_self("Token").with_public_initializer(initializer_call_interface);
    let token_contract_address = token_contract.to_address();
    env.advance_block_by(1);
    (&mut env, token_contract_address, owner, recipient)
}

pub fn setup_and_mint(with_account_contracts: bool) -> (&mut TestEnvironment, AztecAddress, AztecAddress, AztecAddress, Field) {
    // Setup
    let (env, token_contract_address, owner, recipient) = setup(with_account_contracts);
    let mint_amount = 10000;
    // Mint some tokens
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    let mint_private_call_interface = Token::at(token_contract_address).mint_private(mint_amount, secret_hash);
    env.call_public(mint_private_call_interface);

    let mint_public_call_interface = Token::at(token_contract_address).mint_public(owner, mint_amount);
    env.call_public(mint_public_call_interface);

    // Time travel so we can read keys from the registry
    env.advance_block_by(6);

    // docs:start:txe_test_store_note
    // Store a note in the cache so we can redeem it
    env.store_note_in_cache(
        &mut TransparentNote::new(mint_amount, secret_hash),
        Token::storage().pending_shields.slot,
        token_contract_address
    );
    // docs:end:txe_test_store_note

    // Redeem our shielded tokens
    let redeem_shield_call_interface = Token::at(token_contract_address).redeem_shield(owner, mint_amount, secret);
    env.call_private_void(redeem_shield_call_interface);

    (env, token_contract_address, owner, recipient, mint_amount)
}

// docs:start:txe_test_read_public
pub fn check_public_balance(token_contract_address: AztecAddress, address: AztecAddress, address_amount: Field) {
    let current_contract_address = get_contract_address();
    cheatcodes::set_contract_address(token_contract_address);
    let block_number = get_block_number();

    let balances_slot = Token::storage().public_balances.slot;
    let address_slot = derive_storage_slot_in_map(balances_slot, address);
    let amount: U128 = storage_read(token_contract_address, address_slot, block_number);
    assert(amount.to_field() == address_amount, "Public balance is not correct");
    cheatcodes::set_contract_address(current_contract_address);
}
// docs:end:txe_test_read_public

// docs:start:txe_test_call_unconstrained
pub fn check_private_balance(token_contract_address: AztecAddress, address: AztecAddress, address_amount: Field) {
    let current_contract_address = get_contract_address();
    cheatcodes::set_contract_address(token_contract_address);
    // Direct call to unconstrained
    let balance_of_private = Token::balance_of_private(address);
    assert(balance_of_private == address_amount, "Private balance is not correct");
    cheatcodes::set_contract_address(current_contract_address);
}
// docs:end:txe_test_call_unconstrained

