use dep::aztec::context::PrivateContext;
use dep::aztec::protocol_types::{
    constants::GENERATOR_INDEX__SIGNATURE_PAYLOAD, hash::pedersen_hash, traits::{Hash, Serialize},
    address::AztecAddress
};

use dep::authwit::entrypoint::function_call::{FunctionCall, FUNCTION_CALL_SIZE_IN_BYTES};

// FUNCTION_CALL_SIZE * DAPP_MAX_CALLS + 1
global DAPP_PAYLOAD_SIZE: u64 = 5;
// FUNCTION_CALL_SIZE_IN_BYTES * DAPP_MAX_CALLS + 32
global DAPP_PAYLOAD_SIZE_IN_BYTES: u64 = 129;

global DAPP_MAX_CALLS: u64 = 1;

// Note: If you change the following struct you have to update default_entrypoint.ts
// docs:start:app-payload-struct
struct DAppPayload {
  function_calls: [FunctionCall; DAPP_MAX_CALLS],
  nonce: Field,
}
// docs:end:app-payload-struct

impl Serialize<DAPP_PAYLOAD_SIZE> for DAppPayload {
  // Serializes the entrypoint struct
  fn serialize(self) -> [Field; DAPP_PAYLOAD_SIZE] {
    let mut fields: BoundedVec<Field, DAPP_PAYLOAD_SIZE> = BoundedVec::new();
    for call in self.function_calls {
      fields.extend_from_array(call.serialize());
    }
    fields.push(self.nonce);
    fields.storage
  }
}

impl Hash for DAppPayload {
  fn hash(self) -> Field {
    pedersen_hash(
      self.serialize(),
      GENERATOR_INDEX__SIGNATURE_PAYLOAD
    )
  }
}

impl DAppPayload {
    // Serializes the payload as an array of bytes. Useful for hashing with sha256.
    fn to_be_bytes(self) -> [u8; DAPP_PAYLOAD_SIZE_IN_BYTES] {
        let mut bytes: BoundedVec<u8, DAPP_PAYLOAD_SIZE_IN_BYTES> = BoundedVec::new();

        for i in 0..DAPP_MAX_CALLS {
            bytes.extend_from_array(self.function_calls[i].to_be_bytes());
        }
        bytes.extend_from_array(self.nonce.to_be_bytes(32));

        bytes.storage
    }

    // Executes all private and public calls
    // docs:start:entrypoint-execute-calls
    fn execute_calls(self, context: &mut PrivateContext, target_address: AztecAddress) {
        for call in self.function_calls {
            // whitelist the calls that the user can do only go to the expected Dapp contract
            assert(call.target_address == target_address);
            if call.is_public {
                context.call_public_function_with_packed_args(
                    call.target_address,
                    call.function_selector,
                    call.args_hash,
                    false,
                    false
                );
            } else {
                let _result = context.call_private_function_with_packed_args(
                    call.target_address,
                    call.function_selector,
                    call.args_hash,
                    false,
                    false
                );
            }
        }
    }
    // docs:end:entrypoint-execute-calls
}
