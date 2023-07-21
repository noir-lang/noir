mod storage;
mod ecdsa_public_key_note;

// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
contract EcdsaAccount {
    use dep::std;
    use dep::aztec::entrypoint;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::context::Context;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::debug_log;
    use dep::aztec::oracle::get_public_key::get_public_key;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::types::vec::BoundedVec;
    use dep::aztec::types::point::Point;

    use dep::aztec::constants_gen::MAX_NOTE_FIELDS_LENGTH;
    use dep::aztec::note::utils::compute_note_hash_and_nullifier;
    use dep::aztec::note::note_header::NoteHeader;

    use crate::storage::Storage;
    use crate::ecdsa_public_key_note::EcdsaPublicKeyNote;
    use crate::ecdsa_public_key_note::EcdsaPublicKeyNoteInterface;
    use crate::ecdsa_public_key_note::ECDSA_PUBLIC_KEY_NOTE_LEN;

    // All calls made by this account will be routed through this entrypoint
    fn entrypoint(  
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64],
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        
        // Initialise context
        // 71 = ENTRYPOINT_PAYLOAD_SIZE(7) + 64
        let mut args: BoundedVec<Field, 71> = BoundedVec::new(0);
        args = args.push_array(payload.serialize());
        for byte in signature { args = args.push(byte as Field); }
        let mut context = Context::new(inputs, abi::hash_args(args.storage));

        // Load public key from storage
        let storage = Storage::init();
        let (context_1, public_key) = storage.public_key.get_note(context);
        context = context_1;

        // Verify payload signature using Ethereum's signing scheme
        let payload_bytes: [u8; entrypoint::ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = payload.to_be_bytes();
        let challenge: [u8; 32] = std::hash::sha256(payload_bytes);
        let verification = std::ecdsa_secp256k1::verify_signature(public_key.x, public_key.y, signature, challenge);
        assert(verification == true);

        // debug_log::debug_log_format("Verification result is {0}", [verification as Field]);
        // debug_log::debug_log_array_with_prefix("public_key.x", public_key.x);
        // debug_log::debug_log_array_with_prefix("public_key.y", public_key.y);
        // debug_log::debug_log_array_with_prefix("challenge", challenge);
        // debug_log::debug_log_array_with_prefix("signature", signature);

        context = payload.execute_calls(context);
        context.finish()
    }

    // Creates a new account out of an ECDSA public key to use for signature verification
    fn constructor(
        inputs: pub PrivateContextInputs,
        signing_pub_key_x: pub [u8;32],
        signing_pub_key_y: pub [u8;32],
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        
        let mut args: BoundedVec<Field, 64> = BoundedVec::new(0);
        for byte in signing_pub_key_x { args = args.push(byte as Field); }
        for byte in signing_pub_key_y { args = args.push(byte as Field); }
        let mut context = Context::new(inputs, abi::hash_args(args.storage));
        
        let this = inputs.call_context.storage_contract_address;
        let pub_key_note = EcdsaPublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        context = storage.public_key.initialise(context, pub_key_note);
        
        context = emit_encrypted_log(
            context,
            this,
            storage.public_key.storage_slot,
            this,
            get_public_key(this),
            pub_key_note.serialise(),
        );

        context.finish()
    }

    /// ABI stev type "unconstrained"
    fn stev(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> pub [Field; 4] {
        assert(storage_slot == 1);
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        compute_note_hash_and_nullifier(EcdsaPublicKeyNoteInterface, note_header, preimage)
    }
}
