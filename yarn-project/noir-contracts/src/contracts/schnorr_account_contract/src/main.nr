mod storage;
mod public_key_note;

// Account contract that uses Schnorr signatures for authentication.
// The signing key is stored in an immutable private note and should be different from the encryption/nullifying key.
contract SchnorrAccount {
    use dep::std;
    use dep::aztec::entrypoint;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::context::PrivateContext;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::get_public_key::get_public_key;
    use dep::aztec::types::vec::BoundedVec;
    use dep::aztec::types::point::Point;
    use dep::aztec::note::utils as note_utils;
    use dep::aztec::note::note_header::NoteHeader;
    use dep::aztec::constants_gen::MAX_NOTE_FIELDS_LENGTH;
    use dep::aztec::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;
    use dep::aztec::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;

    use crate::storage::Storage;
    use crate::public_key_note::PublicKeyNote;
    use crate::public_key_note::PublicKeyNoteMethods;
    use crate::public_key_note::PUBLIC_KEY_NOTE_LEN;

    // docs:start:entrypoint

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64], // schnorr signature of the payload hash
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Initialize context
        // ENTRYPOINT_PAYLOAD_SIZE(13) + 64
        let mut args: BoundedVec<Field, 77> = BoundedVec::new(0);
        args.push_array(payload.serialize());
        for byte in signature { args.push(byte as Field); }
        let mut context = PrivateContext::new(inputs, abi::hash_args(args.storage));

        // Load public key from storage
        let storage = Storage::init();
        let public_key = storage.signing_public_key.get_note(&mut context);

        // Verify payload signature
        let payload_fields: [Field; entrypoint::ENTRYPOINT_PAYLOAD_SIZE] = payload.serialize();
        let message_field: Field = std::hash::pedersen_with_separator(payload_fields, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];
        // TODO workaround for https://github.com/noir-lang/noir/issues/2421
        let message_bytes_slice = message_field.to_be_bytes(32);
        let mut message_bytes: [u8; 32] = [0; 32];
        for i in 0..32 {
          message_bytes[i] = message_bytes_slice[i];
        }

        // Verify signature of the payload bytes
        let verification = std::schnorr::verify_signature(public_key.x, public_key.y, signature, message_bytes);
        assert(verification == true);

        // docs:end:entrypoint

        // Execute calls
        payload.execute_calls(&mut context);

        context.finish()
    }

    // Constructs the contract
    fn constructor(
        inputs: pub PrivateContextInputs,
        signing_pub_key_x: pub Field,
        signing_pub_key_y: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        
        let mut context = PrivateContext::new(inputs, abi::hash_args([signing_pub_key_x, signing_pub_key_y]));
        
        let this = context.this_address();
        let mut pub_key_note = PublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        storage.signing_public_key.initialise(&mut context, &mut pub_key_note);
        
        emit_encrypted_log(
            &mut context,
            this,
            storage.signing_public_key.storage_slot,
            get_public_key(this),
            pub_key_note.serialise(),
        );

        context.finish()
    }

    // Computes notes hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; PUBLIC_KEY_NOTE_LEN]) -> [Field; 4] {
        assert(storage_slot == 1);
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(PublicKeyNoteMethods, note_header, preimage)
    }
}
