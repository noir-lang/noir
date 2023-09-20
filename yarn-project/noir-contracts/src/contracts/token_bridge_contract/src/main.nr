mod util;
mod token_interface;

// Minimal implementation of the token bridge that can move funds between L1 <> L2.
// The bridge has a corresponding Portal contract on L1 that it is attached to 
// And corresponds to a Token on L2 that uses the `AuthWit` accounts pattern.
// Bridge has to be set as a minter on the token before it can be used

contract TokenBridge {
    use dep::aztec::{
        context::{Context},
        state_vars::{public_state::PublicState},
        types::type_serialisation::field_serialisation::{
            FieldSerialisationMethods, FIELD_SERIALISED_LEN,
        },
        types::address::{AztecAddress, EthereumAddress},
        oracle::compute_selector::compute_selector,
    };

    use crate::token_interface::Token;
    use crate::util::{get_mint_public_content_hash, get_mint_private_content_hash, get_withdraw_content_hash, compute_secret_hash};

    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        token: PublicState<Field, 1>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                token: PublicState::new(
                    context,
                    1,
                    FieldSerialisationMethods,
                ),
            }
        }
    }

    // Constructs the contract.
    #[aztec(private)]
    fn constructor() {
        // Currently not possible to execute public calls from constructor as code not yet available to sequencer.
        // let selector = compute_selector("_initialize((Field))");
        // let _callStackItem = context.call_public_function(context.this_address(), selector, [context.msg_sender()]);
    }

    // Consumes a L1->L2 message and calls the token contract to mint the appropriate amount publicly
    #[aztec(public)]
    fn claim_public(
        to: AztecAddress,
        amount: Field,
        canceller: EthereumAddress,
        msg_key: Field,
        secret: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));

        let content_hash = get_mint_public_content_hash(to.address, amount, canceller.address);
        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret);

        // Mint tokens 
        Token::at(storage.token.read()).mint_public(context, to.address, amount);

        1
    }

    // docs:start:exit_to_l1_public
    // Burns the appropriate amount of tokens and creates a L2 to L1 withdraw message publicly
    // Requires `msg.sender` to give approval to the bridge to burn tokens on their behalf using witness signatures 
    #[aztec(public)]
    fn exit_to_l1_public(
        recipient: EthereumAddress, // ethereum address to withdraw to
        amount: Field,
        callerOnL1: EthereumAddress, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
        nonce: Field, // nonce used in the approval message by `msg.sender` to let bridge burn their tokens on L2
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));

        // Send an L2 to L1 message
        let content = get_withdraw_content_hash(recipient.address, amount, callerOnL1.address);
        context.message_portal(content);     

        // Burn tokens 
        Token::at(storage.token.read()).burn_public(context, context.msg_sender(), amount, nonce);
   
        1
    }
    // docs:end:exit_to_l1_public

    // Consumes a L1->L2 message and calls the token contract to mint the appropriate amount in private assets
    // User needs to call token.redeem_shield() to get the private assets
    #[aztec(private)]
    fn claim_private(
        amount: Field,
        secret_hash_for_redeeming_minted_notes: Field, // secret hash used to redeem minted notes at a later time. This enables anyone to call this function and mint tokens to a user on their behalf
        canceller: EthereumAddress,
        msg_key: Field, // L1 to L2 message key as derived from the inbox contract
        secret_for_L1_to_L2_message_consumption: Field, // secret used to consume the L1 to L2 message
    ) -> Field {
        // Consume L1 to L2 message and emit nullifier
        let content_hash = get_mint_private_content_hash(amount, secret_hash_for_redeeming_minted_notes, canceller.address);
        context.consume_l1_to_l2_message(msg_key, content_hash, secret_for_L1_to_L2_message_consumption);

        // Mint tokens on L2 
        // `mint_private` on token is public. So we call an internal public function 
        // which then calls the public method on the token contract.
        // Since the secret_hash is passed, no secret is leaked. 
        context.call_public_function(
            context.this_address(),
            compute_selector("_call_mint_on_token(Field,Field)"),
            [amount, secret_hash_for_redeeming_minted_notes],
        );

        1
    }

    // docs:start:exit_to_l1_private
    // Burns the appropriate amount of tokens and creates a L2 to L1 withdraw message privately
    // Requires `msg.sender` (caller of the method) to give approval to the bridge to burn tokens on their behalf using witness signatures 
    #[aztec(private)]
    fn exit_to_l1_private(
        recipient: EthereumAddress, // ethereum address to withdraw to
        token: AztecAddress,
        amount: Field,
        callerOnL1: EthereumAddress, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
        nonce: Field, // nonce used in the approval message by `msg.sender` to let bridge burn their tokens on L2
    ) -> Field {
        // Send an L2 to L1 message
        let content = get_withdraw_content_hash(recipient.address, amount, callerOnL1.address);
        context.message_portal(content);

        // Assert that user provided token address is same as seen in storage. 
        context.call_public_function(context.this_address(), compute_selector("_assert_token_is_same(Field)"), [token.address]);

        // Burn tokens
        Token::at(token.address).burn(&mut context, context.msg_sender(), amount, nonce);

        1
    }
    /// docs:end:exit_to_l1_private
    
    // /// Unconstrained /// 

    unconstrained fn token() -> Field {
        let storage = Storage::init(Context::none());
        storage.token.read()
    }

    /// SHOULD BE Internal ///

    // We cannot do this from the constructor currently 
    // Since this should be internal, for now, we ignore the safety checks of it, as they are 
    // enforced by it being internal and only called from the constructor.
    #[aztec(public)]
    fn _initialize(token: AztecAddress) {
        let storage = Storage::init(Context::public(&mut context));
        storage.token.write(token.address);
    }

    // This is a public call as we need to read from public storage.
    // Also, note that user hashes their secret in private and only sends the hash in public
    // meaning only user can `redeem_shield` at a later time with their secret.
    #[aztec(public)]
    internal fn _call_mint_on_token(amount: Field, secret_hash: Field){
        let storage = Storage::init(Context::public(&mut context));
        Token::at(storage.token.read()).mint_private(context, amount, secret_hash);
    }

    #[aztec(public)]
    internal fn _assert_token_is_same(token: Field) {
        let storage = Storage::init(Context::public(&mut context));
        assert(storage.token.read() == token, "Token address is not the same as seen in storage");
    }
}
