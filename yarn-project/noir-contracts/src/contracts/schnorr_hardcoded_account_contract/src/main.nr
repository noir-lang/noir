// docs:start:contract
// Account contract that uses Schnorr signatures for authentication using a hardcoded public key.
contract SchnorrHardcodedAccount {
    use dep::std;
    use dep::aztec::{
        entrypoint::{ EntrypointPayload, ENTRYPOINT_PAYLOAD_SIZE },
        abi::{ PrivateCircuitPublicInputs, PrivateContextInputs, Hasher },
        types::{ vec::BoundedVec, point::Point },
        constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD,
        context::PrivateContext,
        account::AccountActions,
        oracle::auth_witness::get_auth_witness,
    };

    global public_key_x: Field = 0x0ede3d33c920df8fdf43f3e39ed38b0882c25b056620ef52fd016fe811aa2443;
    global public_key_y: Field = 0x29155934ffaa105323695b5f91faadd84acc21f4a8bda2fad760f992d692bc7f;

    global ACCOUNT_ACTIONS_STORAGE_SLOT = 1;

    #[aztec(private)]
    fn constructor() {}

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

    // docs:start:is-valid
    #[contract_library_method]
    fn is_valid_impl(_context: &mut PrivateContext, message_hash: Field) -> pub bool {
        // Load auth witness and format as an u8 array
        let witness: [Field; 64] = get_auth_witness(message_hash);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 { signature[i] = witness[i] as u8; }

        // Verify signature using hardcoded public key
        let verification = std::schnorr::verify_signature(public_key_x, public_key_y, signature, message_hash.to_be_bytes(32));
        assert(verification == true);
        true
    }
    // docs:end:is-valid

}
// docs:end:contract