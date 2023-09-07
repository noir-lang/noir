// docs:start:contract
// Account contract that uses Schnorr signatures for authentication using a hardcoded public key.
contract SchnorrHardcodedAccount {
    global public_key_x: Field = 0x0ede3d33c920df8fdf43f3e39ed38b0882c25b056620ef52fd016fe811aa2443;
    global public_key_y: Field = 0x29155934ffaa105323695b5f91faadd84acc21f4a8bda2fad760f992d692bc7f;

    use dep::std;
    use dep::aztec::{
        entrypoint::{ EntrypointPayload, ENTRYPOINT_PAYLOAD_SIZE },
        abi,
        abi::{ PrivateCircuitPublicInputs, PrivateContextInputs, Hasher },
        types::{ vec::BoundedVec, point::Point },
        constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD,
        context::PrivateContext,
    };

    // Entrypoint for authenticating and executing calls from this account
    #[aztec(private)]
    fn entrypoint(
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: pub [u8;64], // schnorr signature of the payload hash
    ) {
        // docs:start:entrypoint-auth
        // Verify payload signature
        let serialised_payload: [Field; ENTRYPOINT_PAYLOAD_SIZE] = payload.serialize();
        let hashed_payload: Field = std::hash::pedersen_with_separator(serialised_payload, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];

        // TODO: Workaround for https://github.com/noir-lang/noir/issues/2421
        let message_bytes_slice = hashed_payload.to_be_bytes(32);
        let mut message_bytes: [u8; 32] = [0; 32];
        for i in 0..32 { message_bytes[i] = message_bytes_slice[i]; }

        let verification = std::schnorr::verify_signature(public_key_x, public_key_y, signature, message_bytes);
        assert(verification == true);
        // docs:end:entrypoint-auth

        // docs:start:entrypoint-exec
        // Execute calls
        payload.execute_calls(&mut context);
        // docs:end:entrypoint-exec
    }

    // Constructs the contract
    #[aztec(private)]
    fn constructor() {}
}
// docs:end:contract