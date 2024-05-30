mod ecdsa_public_key_note;

// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
contract EcdsaAccount {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, NoteHeader, NoteGetterOptions, PrivateContext, PrivateImmutable};

    use dep::aztec::protocol_types::abis::call_context::CallContext;
    use dep::std;

    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness
    };

    use crate::ecdsa_public_key_note::EcdsaPublicKeyNote;

    #[aztec(storage)]
    struct Storage {
        public_key: PrivateImmutable<EcdsaPublicKeyNote>,
    }

    global ACCOUNT_ACTIONS_STORAGE_SLOT = 2;

    // Creates a new account out of an ECDSA public key to use for signature verification
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(signing_pub_key_x: [u8; 32], signing_pub_key_y: [u8; 32]) {
        let this = context.this_address();
        let header = context.get_header();
        let this_npk_m_hash = header.get_npk_m_hash(&mut context, this);
        // Not emitting outgoing for msg_sender here to not have to register keys for the contract through which we
        // deploy this (typically MultiCallEntrypoint). I think it's ok here as I feel the outgoing here is not that
        // important.
        let this_ovpk_m = header.get_ovpk_m(&mut context, this);
        let this_ivpk_m = header.get_ivpk_m(&mut context, this);

        let mut pub_key_note = EcdsaPublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this_npk_m_hash);
        storage.public_key.initialize(&mut pub_key_note, this_ovpk_m, this_ivpk_m);
    }

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts
    #[aztec(private)]
    fn entrypoint(app_payload: AppPayload, fee_payload: FeePayload) {
        let actions = AccountActions::init(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    #[aztec(noinitcheck)]
    fn spend_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::init(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_private_authwit(inner_hash)
    }

    #[aztec(public)]
    fn spend_public_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::init(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_public_authwit(inner_hash)
    }

    #[aztec(private)]
    #[aztec(internal)]
    fn cancel_authwit(outer_hash: Field) {
        context.push_new_nullifier(outer_hash, 0);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn approve_public_authwit(outer_hash: Field) {
        let actions = AccountActions::init(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.approve_public_authwit(outer_hash)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, outer_hash: Field) -> bool {
        // Load public key from storage
        let storage = Storage::init(context);
        let public_key = storage.public_key.get_note();

        // Load auth witness
        let witness: [Field; 64] = get_auth_witness(outer_hash);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }

        // Verify payload signature using Ethereum's signing scheme
        // Note that noir expects the hash of the message/challenge as input to the ECDSA verification.
        let outer_hash_bytes: [u8; 32] = outer_hash.to_be_bytes(32).as_array();
        let hashed_message: [u8; 32] = std::hash::sha256(outer_hash_bytes);
        let verification = std::ecdsa_secp256k1::verify_signature(public_key.x, public_key.y, signature, hashed_message);
        assert(verification == true);

        true
    }
}
