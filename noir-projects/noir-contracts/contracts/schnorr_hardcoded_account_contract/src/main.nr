// docs:start:contract
// Account contract that uses Schnorr signatures for authentication using a hardcoded public key.
contract SchnorrHardcodedAccount {
    use dep::std;
    use dep::aztec::{
        context::PrivateContext,
        protocol_types::{address::AztecAddress, abis::function_selector::FunctionSelector}
    };

    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness
    };

    global public_key_x: Field = 0x0ede3d33c920df8fdf43f3e39ed38b0882c25b056620ef52fd016fe811aa2443;
    global public_key_y: Field = 0x29155934ffaa105323695b5f91faadd84acc21f4a8bda2fad760f992d692bc7f;

    global ACCOUNT_ACTIONS_STORAGE_SLOT = 1;

    #[aztec(private)]
    fn constructor() {}

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts
    #[aztec(private)]
    fn entrypoint(app_payload: pub AppPayload, fee_payload: pub FeePayload) {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    fn spend_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_private_authwit(inner_hash)
    }

    #[aztec(public)]
    fn spend_public_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_public_authwit(inner_hash)
    }

    #[aztec(private)]
    internal fn cancel_authwit(outer_hash: Field) {
        context.push_new_nullifier(outer_hash, 0);
    }

    #[aztec(public)]
    internal fn approve_public_authwit(outer_hash: Field) {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.approve_public_authwit(outer_hash)
    }

    // docs:start:is-valid
    #[contract_library_method]
    fn is_valid_impl(_context: &mut PrivateContext, outer_hash: Field) -> bool {
        // Load auth witness and format as an u8 array
        let witness: [Field; 64] = get_auth_witness(outer_hash);
        let mut signature: [u8; 64] = [0; 64];
        for i in 0..64 {
            signature[i] = witness[i] as u8;
        }

        // Verify signature using hardcoded public key
        let verification = std::schnorr::verify_signature(
            public_key_x,
            public_key_y,
            signature,
            outer_hash.to_be_bytes(32)
        );
        assert(verification == true);
        true
    }
    // docs:end:is-valid
}
// docs:end:contract
