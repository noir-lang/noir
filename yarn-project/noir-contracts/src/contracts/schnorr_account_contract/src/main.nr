mod public_key_note;

// Account contract that uses Schnorr signatures for authentication.
// The signing key is stored in an immutable private note and should be different from the encryption/nullifying key.
contract SchnorrAccount {
    use dep::std;
    use dep::std::option::Option;
    use dep::aztec::{
        constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD,
        context::{PrivateContext, PublicContext},
        entrypoint::{ENTRYPOINT_PAYLOAD_SIZE, EntrypointPayload},
        log::emit_encrypted_log,
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::get_public_key::get_public_key,
        state_vars::immutable_singleton::ImmutableSingleton,
    };

    use crate::public_key_note::{PublicKeyNote, PublicKeyNoteMethods, PUBLIC_KEY_NOTE_LEN};

    struct Storage {
        signing_public_key: ImmutableSingleton<PublicKeyNote, PUBLIC_KEY_NOTE_LEN>,
    }

    impl Storage {
        fn init(private_context: Option<&mut PrivateContext>, _: Option<&mut PublicContext>) -> pub Self {
            Storage {
                signing_public_key: ImmutableSingleton::new(private_context, 1, PublicKeyNoteMethods),
            }
        }
    }

    // docs:start:entrypoint
    #[aztec(private)]
    fn entrypoint(
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64], // schnorr signature of the payload hash
    ) {
        // Load public key from storage
        let storage = Storage::init(Option::some(&mut context), Option::none());
        let public_key = storage.signing_public_key.get_note();

        // Verify payload signature
        let payload_fields: [Field; ENTRYPOINT_PAYLOAD_SIZE] = payload.serialize();
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
    }

    // Constructs the contract
    #[aztec(private)]
    fn constructor(
        signing_pub_key_x: pub Field,
        signing_pub_key_y: pub Field,
    ) {
        let storage = Storage::init(Option::some(&mut context), Option::none());
        
        let this = context.this_address();
        let mut pub_key_note = PublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        storage.signing_public_key.initialise(&mut pub_key_note);
        
        emit_encrypted_log(
            &mut context,
            this,
            storage.signing_public_key.storage_slot,
            get_public_key(this),
            pub_key_note.serialise(),
        );
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
