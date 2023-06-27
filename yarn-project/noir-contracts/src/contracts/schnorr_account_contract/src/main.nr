contract SchnorrAccount {
    use dep::std;
    use dep::aztec3::entrypoint;
    use dep::aztec3::entrypoint::EntrypointPayload;
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::abi::CallContext;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec3::public_call_stack_item::PublicCallStackItem;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::types::vec::BoundedVec;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::oracle::debug_log::debug_log_format;

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload,
        owner: pub [u8;64], // x and y coordinates concatenated
        signature: pub [Field;3],
        partialContractAddress: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new();

        // Decompress signature from 3 fields to 64 bytes
        let mut decompressed_signature: [u8;64] = [0;64];

        let signature_part_0 = signature[0].to_be_bytes(32);
        for i in 0..31 {
            decompressed_signature[i] = signature_part_0[i + 1];
        }
        let signature_part_1 = signature[1].to_be_bytes(32);
        for i in 0..31 {
            decompressed_signature[i + 31] = signature_part_1[i + 1];
        }
        let signature_part_2 = signature[2].to_be_bytes(32);
        for i in 0..2 {
            decompressed_signature[i + 62] = signature_part_2[i + 1];
        }

        // Verify payload signature
        let payload_bytes: [u8; entrypoint::ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = payload.to_be_bytes();
        let payload_hash: [u8; 32] = std::hash::sha256(payload_bytes);

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
        
        
        // TODO: Find out why this signature verification never fails
        let verification = std::schnorr::verify_signature(x, y, decompressed_signature, payload_hash);
        constrain verification == true;

        // TODO: When we can use pedersen with non-zero generators, get this working        
        //let reproduced_address = dep::std::hash::pedersen([owner, partialContractAddress])[0];
        //constrain reproduced_address == inputs.call_context.storage_contract_address;

        for i in 0..entrypoint::ACCOUNT_MAX_PRIVATE_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let mut args = [0; abi::MAX_ARGS];
                for j in 0..abi::MAX_ARGS {
                    args[j] = payload.flattened_args[i * abi::MAX_ARGS + j];
                }
                let (_callStackItem, updated_context) = PrivateCallStackItem::call(target_address, function_selector, args, context);
                context = updated_context;
            }
        }

        for i in entrypoint::ACCOUNT_MAX_PRIVATE_CALLS..entrypoint::ACCOUNT_MAX_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let mut args = [0; abi::MAX_ARGS];
                for j in 0..abi::MAX_ARGS {
                    args[j] = payload.flattened_args[i * abi::MAX_ARGS + j];
                }
                let (_callStackItem, updated_context) = PublicCallStackItem::call(target_address, function_selector, args, context);
                context = updated_context;
            }
        }

        // // Create args_hash manually since we exceed the MAX_ARGS in PrivateFunctionContext.
        // // Alternatively, we could make PrivateFunctionContext generic on the args size.
        // // ENTRYPOINT_PAYLOAD_SIZE + size_of(signature) = 21 + 64 = 85
        // let mut args: BoundedVec<Field, 85> = BoundedVec::new(0);
        // args = args.push_array(payload.flattened_args);
        // args = args.push_array(payload.flattened_selectors);
        // args = args.push(payload.nonce);
        // for byte in signature {
        //     args = args.push(byte as Field);
        // }
        // TODO(#499): Use hash index
        // let args_hash = dep::std::hash::pedersen(args.storage)[0];
        
        // TODO(#754): Pedersen is throwing a panic in wasm when computing the hash for too many args.
        let args_hash = 0;

        // Finish returns the already serialized payload, so we just tweak the corresponding field
        // Horrible, yes. But we need to rethink the private function context api anyway.
        // TODO(#745): Do not store args, set hash directly in context API
        let mut ret = context.finish(inputs);
        ret.args_hash = args_hash;
        ret
    }

    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateFunctionContext::new().finish(inputs)
    }
}
