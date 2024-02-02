// docs:start:token_bridge_imports
mod token_interface;

// Minimal implementation of the token bridge that can move funds between L1 <> L2.
// The bridge has a corresponding Portal contract on L1 that it is attached to
// And corresponds to a Token on L2 that uses the `AuthWit` accounts pattern.
// Bridge has to be set as a minter on the token before it can be used

contract TokenBridge {
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::{
            AztecAddress,
            EthAddress,
        },
    };

    use dep::aztec::{
        context::{Context},
        hash::{compute_secret_hash},
        state_vars::{public_state::PublicState},
    };

    use dep::token_portal_content_hash_lib::{get_mint_public_content_hash, get_mint_private_content_hash, get_withdraw_content_hash};

    use crate::token_interface::Token;
    // docs:end:token_bridge_imports

    // docs:start:token_bridge_storage_and_constructor
    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        token: PublicState<AztecAddress>,
    }

    // Constructs the contract.
    #[aztec(private)]
    fn constructor(token: AztecAddress) {
        let selector = FunctionSelector::from_signature("_initialize((Field))");
        context.call_public_function(context.this_address(), selector, [token.to_field()]);
    }
    // docs:end:token_bridge_storage_and_constructor

    // docs:start:claim_public
    // Consumes a L1->L2 message and calls the token contract to mint the appropriate amount publicly
    #[aztec(public)]
    fn claim_public(to: AztecAddress, amount: Field, canceller: EthAddress, msg_key: Field, secret: Field) {
        let content_hash = get_mint_public_content_hash(to, amount, canceller);

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret);

        // Mint tokens
        Token::at(storage.token.read()).mint_public(context, to, amount);
    }
    // docs:end:claim_public

    // docs:start:exit_to_l1_public
    // Burns the appropriate amount of tokens and creates a L2 to L1 withdraw message publicly
    // Requires `msg.sender` to give approval to the bridge to burn tokens on their behalf using witness signatures
    #[aztec(public)]
    fn exit_to_l1_public(
        recipient: EthAddress, // ethereum address to withdraw to
        amount: Field,
        callerOnL1: EthAddress, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
        nonce: Field // nonce used in the approval message by `msg.sender` to let bridge burn their tokens on L2
    ) {
        // Send an L2 to L1 message
        let content = get_withdraw_content_hash(recipient, amount, callerOnL1);
        context.message_portal(content);

        // Burn tokens
        Token::at(storage.token.read()).burn_public(context, context.msg_sender(), amount, nonce);
    }
    // docs:end:exit_to_l1_public
    // docs:start:claim_private
    // Consumes a L1->L2 message and calls the token contract to mint the appropriate amount in private assets
    // User needs to call token.redeem_shield() to get the private assets
    #[aztec(private)]
    fn claim_private(
        secret_hash_for_redeeming_minted_notes: Field, // secret hash used to redeem minted notes at a later time. This enables anyone to call this function and mint tokens to a user on their behalf
        amount: Field,
        canceller: EthAddress,
        msg_key: Field, // L1 to L2 message key as derived from the inbox contract
        secret_for_L1_to_L2_message_consumption: Field // secret used to consume the L1 to L2 message
    ) {
        // Consume L1 to L2 message and emit nullifier
        let content_hash = get_mint_private_content_hash(secret_hash_for_redeeming_minted_notes, amount, canceller);
        context.consume_l1_to_l2_message(msg_key, content_hash, secret_for_L1_to_L2_message_consumption);

        // Mint tokens on L2
        // `mint_private` on token is public. So we call an internal public function
        // which then calls the public method on the token contract.
        // Since the secret_hash is passed, no secret is leaked.
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("_call_mint_on_token(Field,Field)"),
            [amount, secret_hash_for_redeeming_minted_notes]
        );
    }
    // docs:end:claim_private

    // docs:start:exit_to_l1_private
    // Burns the appropriate amount of tokens and creates a L2 to L1 withdraw message privately
    // Requires `msg.sender` (caller of the method) to give approval to the bridge to burn tokens on their behalf using witness signatures
    #[aztec(private)]
    fn exit_to_l1_private(
        token: AztecAddress,
        recipient: EthAddress, // ethereum address to withdraw to
        amount: Field,
        callerOnL1: EthAddress, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
        nonce: Field // nonce used in the approval message by `msg.sender` to let bridge burn their tokens on L2
    ) {
        // Send an L2 to L1 message
        let content = get_withdraw_content_hash(recipient, amount, callerOnL1);
        context.message_portal(content);

        // docs:start:call_assert_token_is_same
        // Assert that user provided token address is same as seen in storage.
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("_assert_token_is_same((Field))"),
            [token.to_field()]
        );
        // docs:end:call_assert_token_is_same

        // Burn tokens
        Token::at(token).burn(&mut context, context.msg_sender(), amount, nonce);
    }
    /// docs:end:exit_to_l1_private

    // View function that is callable by other contracts.
    // Unconstrained can't be called by others since it isn't safe.
    #[aztec(public)]
    fn get_token() -> AztecAddress {
        storage.token.read()
    }

    // /// Unconstrained ///

    // docs:start:read_token
    unconstrained fn token() -> pub AztecAddress {
        storage.token.read()
    }
    // docs:end:read_token

    #[aztec(public)]
    internal fn _initialize(token: AztecAddress) {
        storage.token.write(token);
    }

    // docs:start:call_mint_on_token
    // This is a public call as we need to read from public storage.
    // Also, note that user hashes their secret in private and only sends the hash in public
    // meaning only user can `redeem_shield` at a later time with their secret.
    #[aztec(public)]
    internal fn _call_mint_on_token(amount: Field, secret_hash: Field) {
        Token::at(storage.token.read()).mint_private(context, amount, secret_hash);
    }
    // docs:end:call_mint_on_token

    // docs:start:assert_token_is_same
    #[aztec(public)]
    internal fn _assert_token_is_same(token: AztecAddress) {
        assert(storage.token.read().eq(token), "Token address is not the same as seen in storage");
    }
    // docs:end:assert_token_is_same

    // TODO: remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented
    // docs:start:compute_note_hash_and_nullifier_placeholder
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; 0]
    ) -> pub [Field; 4] {
        [0, 0, 0, 0]
    }
    // docs:end:compute_note_hash_and_nullifier_placeholder
}
