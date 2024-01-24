mod ecdsa_public_key_note;

// Account contract that uses ECDSA signatures for authentication on the same curve as Ethereum.
// The signing key is stored in an immutable private note and should be different from the signing key.
contract EcdsaAccount {
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::std;
    use dep::std::option::Option;
    use dep::aztec::{
        abi::CallContext,
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::get_public_key::get_public_key,
        state_vars::immutable_singleton::ImmutableSingleton,
    };
    use dep::authwit:: {
        entrypoint::{ EntrypointPayload, ENTRYPOINT_PAYLOAD_SIZE },
        account::AccountActions,
        auth_witness::get_auth_witness,
    };

    use crate::ecdsa_public_key_note::{
        EcdsaPublicKeyNote, EcdsaPublicKeyNoteInterface, ECDSA_PUBLIC_KEY_NOTE_LEN,
    };

    struct Storage {
        public_key: ImmutableSingleton<EcdsaPublicKeyNote, ECDSA_PUBLIC_KEY_NOTE_LEN>,
    }

    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                public_key: ImmutableSingleton::new(context, 1, EcdsaPublicKeyNoteInterface),
            }
        }
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
    fn entrypoint(payload: pub EntrypointPayload) {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.entrypoint(payload);
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

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]
    ) -> pub [Field; 4] {
        assert(storage_slot == 1);
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(EcdsaPublicKeyNoteInterface, note_header, serialized_note)
    }
}
