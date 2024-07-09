// docs:start:contract
// Account contract that uses Schnorr signatures for authentication using a hardcoded public key.
contract SchnorrHardcodedAccount {
    use dep::std;
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, PrivateContext};

    use dep::authwit::{
        entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions,
        auth_witness::get_auth_witness
    };

    global public_key_x: Field = 0x16b93f4afae55cab8507baeb8e7ab4de80f5ab1e9e1f5149bf8cd0d375451d90;
    global public_key_y: Field = 0x208d44b36eb6e73b254921134d002da1a90b41131024e3b1d721259182106205;

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts
    #[aztec(private)]
    fn entrypoint(app_payload: AppPayload, fee_payload: FeePayload) {
        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    #[aztec(view)]
    fn verify_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::init(&mut context, is_valid_impl);
        actions.verify_private_authwit(inner_hash)
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
        let verification = std::schnorr::verify_signature_slice(
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
