mod storage;
mod public_key_note;

// Account contract that uses Schnorr signatures for authentication.
// The signing key is stored in an immutable private note and should be different from the encryption/nullifying key.
contract SchnorrMultiKeyAccount {
    use dep::std;
    use dep::aztec::entrypoint;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::context::Context;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::get_public_key::get_public_key;
    use dep::aztec::types::vec::BoundedVec;
    use dep::aztec::types::point::Point;
    use dep::aztec::oracle::debug_log::debug_log_format;
    use dep::aztec::note::utils::compute_note_hash_and_nullifier;
    use dep::aztec::note::note_header::NoteHeader;
    use dep::aztec::constants_gen::MAX_NOTE_FIELDS_LENGTH;
    use dep::aztec::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;

    use crate::storage::Storage;
    use crate::public_key_note::PublicKeyNote;
    use crate::public_key_note::PublicKeyNoteInterface;
    use crate::public_key_note::PUBLIC_KEY_NOTE_LEN;

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64], // schnorr signature of the payload hash
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Initialize context
        // 71 = ENTRYPOINT_PAYLOAD_SIZE(7) + 64
        let mut args: BoundedVec<Field, 71> = BoundedVec::new(0);
        args = args.push_array(payload.serialize());
        for byte in signature { args = args.push(byte as Field); }
        let mut context = Context::new(inputs, abi::hash_args(args.storage));

        // Load public key from storage
        let storage = Storage::init();
        let (context_1, public_key) = storage.signing_public_key.get_note(context);
        context = context_1;

        // Verify payload signature
        // TODO: Use pedersen to make the payload hash cheaper to compute
        let payload_bytes: [u8; entrypoint::ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = payload.to_be_bytes();
        let payload_hash: [u8; 32] = std::hash::sha256(payload_bytes);

        // Verify signature of the payload hash
        let verification = std::schnorr::verify_signature(public_key.x, public_key.y, signature, payload_hash);
        assert(verification == true);

        // Execute calls
        context = payload.execute_calls(context);

        context.finish()
    }

    // Constructs the contract
    fn constructor(
        inputs: pub PrivateContextInputs,
        signing_pub_key_x: pub Field,
        signing_pub_key_y: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        
        let mut context = Context::new(inputs, abi::hash_args([signing_pub_key_x, signing_pub_key_y]));
        
        let this = inputs.call_context.storage_contract_address;
        let pub_key_note = PublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        context = storage.signing_public_key.initialise(context, pub_key_note);
        
        context = emit_encrypted_log(
            context,
            this,
            storage.signing_public_key.storage_slot,
            this,
            get_public_key(this),
            pub_key_note.serialise(),
        );

        context.finish()
    }

    /// ABI stev type "unconstrained"
    fn stev(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; PUBLIC_KEY_NOTE_LEN]) -> pub [Field; 4] {
        assert(storage_slot == 1);
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        compute_note_hash_and_nullifier(PublicKeyNoteInterface, note_header, preimage)
    }
}
