mod public_key_note;

// Account contract that uses Schnorr signatures for authentication.
// The signing key is stored in an immutable private note and should be different from the encryption/nullifying key.
contract SchnorrAccount {
    use dep::std;

    use dep::aztec::prelude::{AztecAddress, FunctionSelector, NoteHeader, PrivateContext, PrivateImmutable};
    use dep::aztec::state_vars::{Map, PublicMutable};
    use dep::aztec::{context::Context, oracle::get_public_key::get_public_key};
    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness
    };
    use dep::aztec::hash::compute_siloed_nullifier;
    use dep::aztec::oracle::get_nullifier_membership_witness::get_low_nullifier_membership_witness;

    use crate::public_key_note::{PublicKeyNote, PUBLIC_KEY_NOTE_LEN};

    struct Storage {
        // docs:start:storage
        signing_public_key: PrivateImmutable<PublicKeyNote>,
        // docs:end:storage
        approved_actions: Map<Field, PublicMutable<bool>>,
    }

    // Constructs the contract
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(signing_pub_key_x: pub Field, signing_pub_key_y: pub Field) {
        let this = context.this_address();
        // docs:start:initialize
        let mut pub_key_note = PublicKeyNote::new(signing_pub_key_x, signing_pub_key_y, this);
        storage.signing_public_key.initialize(&mut pub_key_note, true);
        // docs:end:initialize
    }

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts file
    #[aztec(private)]
    fn entrypoint(app_payload: pub AppPayload, fee_payload: pub FeePayload) {
        let actions = AccountActions::private(
            &mut context,
            storage.approved_actions.storage_slot,
            is_valid_impl
        );
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    fn spend_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::private(
            &mut context,
            storage.approved_actions.storage_slot,
            is_valid_impl
        );
        actions.spend_private_authwit(inner_hash)
    }

    #[aztec(public)]
    fn spend_public_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::public(
            &mut context,
            storage.approved_actions.storage_slot,
            is_valid_impl
        );
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
        let actions = AccountActions::public(
            &mut context,
            storage.approved_actions.storage_slot,
            is_valid_impl
        );
        actions.approve_public_authwit(outer_hash)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, outer_hash: Field) -> bool {
        // docs:start:entrypoint
        // Load public key from storage
        let storage = Storage::init(Context::private(context));
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
        let verification = std::schnorr::verify_signature(
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
    * @notice Helper function to check the existing and validity of authwitnesses
    * @dev TODO: myself and block_number should be removed and passed from a context
    * @param myself The address of the contract
    * @param block_number The block number to check the nullifier against
    * @param check_private Whether to check the validity of the authwitness in private state or not
    * @param message_hash The message hash of the message to check the validity
    * @return An array of two booleans, the first is the validity of the authwitness in the private state, 
    * the second is the validity of the authwitness in the public state
    * Both values will be `false` if the nullifier is spent 
    */
    unconstrained fn lookup_validity(
        myself: AztecAddress,
        block_number: u32,
        check_private: bool,
        message_hash: Field
    ) -> pub [bool; 2] {
        let valid_in_private = if check_private {
            let public_key = storage.signing_public_key.view_note();
            let witness: [Field; 64] = get_auth_witness(message_hash);
            let mut signature: [u8; 64] = [0; 64];
            for i in 0..64 {
                signature[i] = witness[i] as u8;
            }
            std::schnorr::verify_signature(
                public_key.x,
                public_key.y,
                signature,
                message_hash.to_be_bytes(32)
            )
        } else {
            false
        };

        let valid_in_public = storage.approved_actions.at(message_hash).read();

        // Compute the nullifier and check if it is spent
        // This will BLINDLY TRUST the oracle, but the oracle is us, and 
        // it is not as part of execution of the contract, so we are good.
        let siloed_nullifier = compute_siloed_nullifier(myself, message_hash);
        let lower_wit = get_low_nullifier_membership_witness(block_number, siloed_nullifier);
        let is_spent = lower_wit.leaf_preimage.nullifier == siloed_nullifier;

        if is_spent {
            [false, false]
        } else {
            [valid_in_private, valid_in_public]
        }
    }
}
