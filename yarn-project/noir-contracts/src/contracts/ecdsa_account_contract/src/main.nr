mod ecdsa_public_key_note;

// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
contract EcdsaAccount {
    use dep::std;
    use dep::std::option::Option;
    use dep::aztec::{
        abi::CallContext,
        constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD,
        context::{PrivateContext, PublicContext, Context},
        entrypoint::{EntrypointPayload, ENTRYPOINT_PAYLOAD_SIZE},
        log::emit_encrypted_log,
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::get_public_key::get_public_key,
        state_vars::immutable_singleton::ImmutableSingleton,
    };

    use crate::ecdsa_public_key_note::{
        EcdsaPublicKeyNote, EcdsaPublicKeyNoteInterface, ECDSA_PUBLIC_KEY_NOTE_LEN,
    };

    struct Storage {
        public_key: ImmutableSingleton<EcdsaPublicKeyNote, ECDSA_PUBLIC_KEY_NOTE_LEN>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                public_key: ImmutableSingleton::new(context, 1, EcdsaPublicKeyNoteInterface),
            }
        }
    }

    // All calls made by this account will be routed through this entrypoint
    #[aztec(private)]
    fn entrypoint(  
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64],
    ) {
        // Load public key from storage
        let storage = Storage::init(Context::private(&mut context));
        let public_key = storage.public_key.get_note();

        // Verify payload signature using Ethereum's signing scheme
        // Note that noir expects the hash of the message/challenge as input to the ECDSA verification.
        let payload_fields: [Field; ENTRYPOINT_PAYLOAD_SIZE] = payload.serialize();
        let message_field: Field = std::hash::pedersen_with_separator(payload_fields, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];
        // TODO workaround for https://github.com/noir-lang/noir/issues/2421
        let message_bytes_slice = message_field.to_be_bytes(32);
        let mut message_bytes: [u8; 32] = [0; 32];
        for i in 0..32 {
          message_bytes[i] = message_bytes_slice[i];
        }
        let hashed_message: [u8; 32] = std::hash::sha256(message_bytes);
        let verification = std::ecdsa_secp256k1::verify_signature(public_key.x, public_key.y, signature, hashed_message);
        assert(verification == true);

        payload.execute_calls(&mut context);
    }


    // Creates a new account out of an ECDSA public key to use for signature verification
    #[aztec(private)]
    fn constructor(
        signing_pub_key_x: pub [u8;32],
        signing_pub_key_y: pub [u8;32],
    ) {
        let storage = Storage::init(Context::private(&mut context));
        
        let this = context.this_address();
        let mut pub_key_note = EcdsaPublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        storage.public_key.initialise(&mut pub_key_note);
        
        emit_encrypted_log(
            &mut context,
            this,
            storage.public_key.storage_slot,
            get_public_key(this),
            pub_key_note.serialise(),
        );
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> [Field; 4] {
        assert(storage_slot == 1);
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(EcdsaPublicKeyNoteInterface, note_header, preimage)
    }
}
