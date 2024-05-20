use dep::aztec::prelude::PrivateContext;
use dep::aztec::protocol_types::{constants::GENERATOR_INDEX__SIGNATURE_PAYLOAD, hash::pedersen_hash, traits::{Hash, Serialize}};

use crate::entrypoint::function_call::{FunctionCall, FUNCTION_CALL_SIZE_IN_BYTES};

// FUNCTION_CALL_SIZE * ACCOUNT_MAX_CALLS + 1
global APP_PAYLOAD_SIZE: u64 = 21;
// FUNCTION_CALL_SIZE_IN_BYTES * ACCOUNT_MAX_CALLS + 32
global APP_PAYLOAD_SIZE_IN_BYTES: u64 = 424;

global ACCOUNT_MAX_CALLS: u64 = 4;

// Note: If you change the following struct you have to update default_entrypoint.ts
// docs:start:app-payload-struct
struct AppPayload {
  function_calls: [FunctionCall; ACCOUNT_MAX_CALLS],
  nonce: Field,
}
// docs:end:app-payload-struct

impl Serialize<APP_PAYLOAD_SIZE> for AppPayload {
  // Serializes the entrypoint struct
  fn serialize(self) -> [Field; APP_PAYLOAD_SIZE] {
    let mut fields: BoundedVec<Field, APP_PAYLOAD_SIZE> = BoundedVec::new();
    for call in self.function_calls {
      fields.extend_from_array(call.serialize());
    }
    fields.push(self.nonce);
    fields.storage
  }
}

impl Hash for AppPayload {
  fn hash(self) -> Field {
    pedersen_hash(
      self.serialize(),
      GENERATOR_INDEX__SIGNATURE_PAYLOAD
    )
  }
}

impl AppPayload {
    // Serializes the payload as an array of bytes. Useful for hashing with sha256.
    fn to_be_bytes(self) -> [u8; APP_PAYLOAD_SIZE_IN_BYTES] {
        let mut bytes: BoundedVec<u8, APP_PAYLOAD_SIZE_IN_BYTES> = BoundedVec::new();

        for i in 0..ACCOUNT_MAX_CALLS {
            bytes.extend_from_array(self.function_calls[i].to_be_bytes());
        }
        bytes.extend_from_slice(self.nonce.to_be_bytes(32));

        bytes.storage
    }

    // Executes all private and public calls
    // docs:start:entrypoint-execute-calls
    fn execute_calls(self, context: &mut PrivateContext) {
        for call in self.function_calls {
            if !call.target_address.is_zero() {
                if call.is_public {
                    context.call_public_function_with_packed_args(
                        call.target_address,
                        call.function_selector,
                        call.args_hash,
                        call.is_static,
                        false
                    );
                } else {
                    let _result = context.call_private_function_with_packed_args(
                        call.target_address,
                        call.function_selector,
                        call.args_hash,
                        call.is_static,
                        false
                    );
                }
            }
        }
    }
    // docs:end:entrypoint-execute-calls
}
