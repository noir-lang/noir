mod public_key_note;

// Account contract that uses Schnorr signatures for authentication.
// The signing key is stored in an immutable private note and should be different from the encryption/nullifying key.
contract SchnorrAccount {
    use dep::std;

    use dep::aztec::prelude::{AztecAddress, FunctionSelector, NoteHeader, PrivateContext, PrivateImmutable};
    use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note;
    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness, auth::{compute_authwit_nullifier, compute_outer_authwit_hash}
    };
    use dep::aztec::hash::compute_siloed_nullifier;
    use dep::aztec::oracle::get_nullifier_membership_witness::get_low_nullifier_membership_witness;

    use crate::public_key_note::{PublicKeyNote, PUBLIC_KEY_NOTE_LEN};

    #[aztec(storage)]
    struct Storage {
        // docs:start:storage
        signing_public_key: PrivateImmutable<PublicKeyNote>,
        // docs:end:storage
    }

    // Constructs the contract
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(signing_pub_key_x: Field, signing_pub_key_y: Field) {
        let this = context.this_address();
        let header = context.get_header();
        let this_npk_m_hash = header.get_npk_m_hash(&mut context, this);
        // Not emitting outgoing for msg_sender here to not have to register keys for the contract through which we
        // deploy this (typically MultiCallEntrypoint). I think it's ok here as I feel the outgoing here is not that
        // important.

        // docs:start:initialize
        let mut pub_key_note = PublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this_npk_m_hash);
        storage.signing_public_key.initialize(&mut pub_key_note).emit(encode_and_encrypt_note(&mut context, this, this));
        // docs:end:initialize
    }

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts file
    #[aztec(private)]
    #[aztec(noinitcheck)]
    fn entrypoint(app_payload: AppPayload, fee_payload: FeePayload) {
        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    #[aztec(noinitcheck)]
    #[aztec(view)]
    fn verify_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.verify_private_authwit(inner_hash)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, outer_hash: Field) -> bool {
        // docs:start:entrypoint
        // Load public key from storage
        let storage = Storage::init(context);
        // docs:start:get_note
        let public_key = storage.signing_public_key.get_note();
        // docs:end:get_note
        // Load auth witness
        let witness: [Field; 64] = get_auth_witness(outer_hash);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }

        // Verify signature of the payload bytes
        let verification = std::schnorr::verify_signature_slice(
            public_key.x,
            public_key.y,
            signature,
            outer_hash.to_be_bytes(32)
        );
        assert(verification == true);
        // docs:end:entrypoint
        true
    }

    /**
    * @notice Helper function to check validity of private authwitnesses
    * @param consumer The address of the consumer of the message
    * @param message_hash The message hash of the message to check the validity
    * @return True if the message_hash can be consumed, false otherwise
    */
    unconstrained fn lookup_validity(consumer: AztecAddress, inner_hash: Field) -> pub bool {
        let public_key = storage.signing_public_key.view_note();

        let message_hash = compute_outer_authwit_hash(consumer, context.chain_id(), context.version(), inner_hash);

        let witness: [Field; 64] = get_auth_witness(message_hash);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }
        let valid_in_private = std::schnorr::verify_signature_slice(
            public_key.x,
            public_key.y,
            signature,
            message_hash.to_be_bytes(32)
        );

        // Compute the nullifier and check if it is spent
        // This will BLINDLY TRUST the oracle, but the oracle is us, and
        // it is not as part of execution of the contract, so we are good.
        let nullifier = compute_authwit_nullifier(context.this_address(), inner_hash);
        let siloed_nullifier = compute_siloed_nullifier(consumer, nullifier);
        let lower_wit = get_low_nullifier_membership_witness(context.block_number(), siloed_nullifier);
        let is_spent = lower_wit.leaf_preimage.nullifier == siloed_nullifier;

        !is_spent & valid_in_private
    }
}
