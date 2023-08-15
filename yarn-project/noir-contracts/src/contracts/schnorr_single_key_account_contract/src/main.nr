// Account contract that uses Schnorr signatures for authentication. The signing key is the same as the
// encryption key, and as such is not stored in the contract but part of the address preimage, so it can
// be verified by passing in the partial address.
contract SchnorrSingleKeyAccount {
    use dep::std;
    use dep::aztec::entrypoint;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::context::PrivateContext;
    use dep::aztec::types::vec::BoundedVec;
    use dep::aztec::types::point::Point;
    use dep::aztec::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;
    use dep::aztec::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        owner: pub [u8;64], // pubkey x and y coordinates concatenated
        signature: pub [u8;64], // schnorr signature of the payload hash
        partial_address: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Initialize context
        // ENTRYPOINT_PAYLOAD_SIZE(13) + 64 + 64 + 1
        let mut args: BoundedVec<Field, 142> = BoundedVec::new(0);
        args.push_array(payload.serialize());
        for byte in owner { args.push(byte as Field); }
        for byte in signature { args.push(byte as Field); }
        args.push(partial_address);
        let mut context = PrivateContext::new(inputs, abi::hash_args(args.storage));

        // Verify payload signature
        let payload_fields: [Field; entrypoint::ENTRYPOINT_PAYLOAD_SIZE] = payload.serialize();
        let message_field: Field = std::hash::pedersen_with_separator(payload_fields, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];
        let message_bytes = message_field.to_be_bytes(32);

        // Convert owner pubkey into fields
        let mut x: Field = 0;
        let mut y: Field = 0;
        let mut mul: Field = 1;
        for i in 0..32 {
          let bytex: Field = owner[31 - i] as Field;
          x = x + (bytex * mul);
          let bytey: Field = owner[63 - i] as Field;
          y = y + (bytey * mul);
          mul *= 256;
        }
        
        // Verify signature of the payload hash
        // TODO: Find out why this signature verification never fails
        let verification = std::schnorr::verify_signature(x, y, signature, message_bytes);
        assert(verification == true);

        // Verify public key against address
        let reproduced_address = dep::std::hash::pedersen_with_separator([x, y, partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)[0];
        assert(reproduced_address == context.this_address());

        // Execute calls
        payload.execute_calls(&mut context);

        context.finish()
    }

    // Constructs the contract
    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateContext::new(inputs, 0).finish()
    }
}
