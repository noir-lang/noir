// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
use dep::aztec::macros::aztec;

#[aztec]
pub contract EcdsaKAccount {
    use dep::aztec::{
        authwit::{account::AccountActions, entrypoint::{app::AppPayload, fee::FeePayload}},
        context::PrivateContext,
        macros::{functions::{initializer, noinitcheck, private, view}, storage::storage},
        messages::message_delivery::MessageDelivery,
        oracle::{auth_witness::get_auth_witness, notes::{get_sender_for_tags, set_sender_for_tags}},
        state_vars::PrivateImmutable,
    };

    use dep::ecdsa_public_key_note::EcdsaPublicKeyNote;

    #[storage]
    struct Storage<Context> {
        signing_public_key: PrivateImmutable<EcdsaPublicKeyNote, Context>,
    }

    // Creates a new account out of an ECDSA public key to use for signature verification
    #[private]
    #[initializer]
    fn constructor(signing_pub_key_x: [u8; 32], signing_pub_key_y: [u8; 32]) {
        let this = context.this_address();
        let pub_key_note = EcdsaPublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);

        // Safety: The sender for tags is only used to compute unconstrained shared secrets for emitting logs.
        // Since this value is only used for unconstrained tagging and not for any constrained logic,
        // it is safe to load from an unconstrained context.
        // TODO(#15752): Improve the sender_for_tags handling here when the original sender is undefined.
        let original_sender = unsafe { get_sender_for_tags().unwrap_or(this) };

        // We set the sender for tags to this contract because we don't want to force the user corresponding to this
        // account to add the account deployer as a sender to their PXE. By setting it to this contract, user's PXE
        // will manage to find the note even if the account deployer is not registered as a sender (i.e
        // `pxe.registerSender(accountDeployer)` was not called)

        // Safety: Comment from above applies here as well.
        unsafe { set_sender_for_tags(this) };
        storage.signing_public_key.initialize(pub_key_note).emit(
            &mut context,
            this,
            MessageDelivery.CONSTRAINED_ONCHAIN,
        );
        // Safety: Comment from above applies here as well.
        unsafe { set_sender_for_tags(original_sender) };
    }

    // Note: If you globally change the entrypoint signature don't forget to update account_entrypoint.ts file
    // using noinitcheck is an optimization, it reduces gates by omitting a check that the contract has been initialized
    #[private]
    #[noinitcheck]
    fn entrypoint(app_payload: AppPayload, fee_payload: FeePayload, cancellable: bool) {
        // Safety: The sender for tags is only used to compute unconstrained shared secrets for emitting logs.
        // Since this value is only used for unconstrained tagging and not for any constrained logic,
        // it is safe to set from a constrained context.
        unsafe { set_sender_for_tags(context.this_address()) };

        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload, cancellable);
    }

    #[private]
    #[noinitcheck]
    #[view]
    fn verify_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.verify_private_authwit(inner_hash)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, outer_hash: Field) -> bool {
        // Load public key from storage
        let storage = Storage::init(context);
        let public_key = storage.signing_public_key.get_note();

        // Load auth witness
        // Safety: The witness is only used as a "magical value" that makes the signature verification below pass.
        // Hence it's safe.
        let witness: [Field; 64] = unsafe { get_auth_witness(outer_hash) };
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }

        // Verify payload signature using Ethereum's signing scheme
        // Note that noir expects the hash of the message/challenge as input to the ECDSA verification.
        let outer_hash_bytes: [u8; 32] = outer_hash.to_be_bytes();
        let hashed_message: [u8; 32] = sha256::digest(outer_hash_bytes);
        std::ecdsa_secp256k1::verify_signature(
            public_key.x,
            public_key.y,
            signature,
            hashed_message,
        )
    }
}
