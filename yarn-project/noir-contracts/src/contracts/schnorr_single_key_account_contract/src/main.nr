// Account contract that uses Schnorr signatures for authentication. The signing key is the same as the
// encryption key, and as such is not stored in the contract but part of the address preimage, so it can
// be verified by passing in the partial contract address.
contract SchnorrSingleKeyAccount {
    use dep::std;
    use dep::aztec::entrypoint;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::context::Context;
    use dep::aztec::types::vec::BoundedVec;
    use dep::aztec::types::point::Point;
    use dep::aztec::oracle::debug_log::debug_log_format;
    use dep::aztec::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        owner: pub [u8;64], // pubkey x and y coordinates concatenated
        signature: pub [u8;64], // schnorr signature of the payload hash
        partial_address: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Initialize context
        // 136 = ENTRYPOINT_PAYLOAD_SIZE(7) + 64 + 64 + 1
        let mut args: BoundedVec<Field, 136> = BoundedVec::new(0);
        args = args.push_array(payload.serialize());
        for byte in owner { args = args.push(byte as Field); }
        for byte in signature { args = args.push(byte as Field); }
        args = args.push(partial_address);
        let mut context = Context::new(inputs, abi::hash_args(args.storage));

        // Verify payload signature
        // TODO: Use pedersen to make the payload hash cheaper to compute
        let payload_bytes: [u8; entrypoint::ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = payload.to_be_bytes();
        let payload_hash: [u8; 32] = std::hash::sha256(payload_bytes);

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
        let verification = std::schnorr::verify_signature(x, y, signature, payload_hash);
        assert(verification == true);

        // Verify public key against address
        let reproduced_address = dep::std::hash::pedersen_with_separator([x, y, partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)[0];
        assert(reproduced_address == inputs.call_context.storage_contract_address);

        // Execute calls
        context = payload.execute_calls(context);

        context.finish()
    }

    // Constructs the contract
    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }
}
