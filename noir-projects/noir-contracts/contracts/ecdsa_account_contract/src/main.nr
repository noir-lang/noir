mod ecdsa_public_key_note;

// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
contract EcdsaAccount {
    use dep::aztec::protocol_types::{abis::call_context::CallContext, address::AztecAddress};
    use dep::std;
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{note_header::NoteHeader, utils as note_utils}, oracle::get_public_key::get_public_key,
        state_vars::PrivateImmutable
    };
    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness
    };

    use crate::ecdsa_public_key_note::EcdsaPublicKeyNote;

    struct Storage {
        public_key: PrivateImmutable<EcdsaPublicKeyNote>,
    }

    global ACCOUNT_ACTIONS_STORAGE_SLOT = 2;

    // Creates a new account out of an ECDSA public key to use for signature verification
    #[aztec(private)]
    fn constructor(signing_pub_key_x: pub [u8; 32], signing_pub_key_y: pub [u8; 32]) {
        let this = context.this_address();
        let mut pub_key_note = EcdsaPublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        storage.public_key.initialize(&mut pub_key_note, true);
    }

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts
    #[aztec(private)]
    fn entrypoint(app_payload: pub AppPayload, fee_payload: pub FeePayload) {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    fn is_valid(message_hash: Field) -> Field {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.is_valid(message_hash)
    }

    #[aztec(public)]
    fn is_valid_public(message_hash: Field) -> Field {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.is_valid_public(message_hash)
    }

    #[aztec(public)]
    internal fn set_is_valid_storage(message_hash: Field, value: bool) {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.internal_set_is_valid_storage(message_hash, value)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, message_field: Field) -> pub bool {
        // Load public key from storage
        let storage = Storage::init(Context::private(context));
        let public_key = storage.public_key.get_note();

        // Load auth witness
        let witness: [Field; 64] = get_auth_witness(message_field);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }

        // Verify payload signature using Ethereum's signing scheme
        // Note that noir expects the hash of the message/challenge as input to the ECDSA verification.
        let hashed_message: [u8; 32] = std::hash::sha256(message_field.to_be_bytes(32));
        let verification = std::ecdsa_secp256k1::verify_signature(public_key.x, public_key.y, signature, hashed_message);
        assert(verification == true);

        true
    }
}
