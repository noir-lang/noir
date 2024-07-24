use dep::aztec::{
    hash::compute_secret_hash, prelude::AztecAddress,
    test::helpers::{cheatcodes, test_environment::TestEnvironment},
    protocol_types::storage::map::derive_storage_slot_in_map,
    note::{note_getter::{MAX_NOTES_PER_PAGE, view_notes}, note_viewer_options::NoteViewerOptions},
    oracle::{execution::get_contract_address, unsafe_rand::unsafe_rand, storage::storage_read},
    context::PrivateContext
};

use crate::{types::{token_note::TokenNote}, PrivateToken};

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
    let initializer_call_interface = PrivateToken::interface().constructor(
        owner,
        "TestToken0000000000000000000000",
        "TT00000000000000000000000000000",
        18
    );
    let token_contract = env.deploy_self("PrivateToken").with_public_initializer(initializer_call_interface);
    let token_contract_address = token_contract.to_address();
    env.advance_block_by(6);
    (&mut env, token_contract_address, owner, recipient)
}

pub fn setup_and_mint(with_account_contracts: bool) -> (&mut TestEnvironment, AztecAddress, AztecAddress, AztecAddress, Field) {
    // Setup
    let (env, token_contract_address, owner, recipient) = setup(with_account_contracts);
    let mint_amount = 10_000;
    let mint_private_call_interface = PrivateToken::at(token_contract_address).privately_mint_private_note(mint_amount);
    env.call_private_void(mint_private_call_interface);
    env.advance_block_by(6);

    check_private_balance(&mut env.private(), token_contract_address, owner, mint_amount);

    (env, token_contract_address, owner, recipient, mint_amount)
}

pub fn check_private_balance(
    context: &mut PrivateContext,
    token_contract_address: AztecAddress,
    address: AztecAddress,
    address_amount: Field
) {
    let current_contract_address = get_contract_address();
    cheatcodes::set_contract_address(token_contract_address);

    let header = context.get_header();
    let owner_npk_m_hash = header.get_npk_m_hash(context, address);

    let balance_of_private = PrivateToken::balance_of_unconstrained(owner_npk_m_hash);
    assert(balance_of_private == address_amount, "Private balance is not correct");
    cheatcodes::set_contract_address(current_contract_address);
}
