use crate::abi;
use crate::types::vec::BoundedVec;
use crate::context::PrivateContext;
use crate::private_call_stack_item::PrivateCallStackItem;
use crate::public_call_stack_item::PublicCallStackItem;    
use crate::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;

use dep::std::hash;

global ACCOUNT_MAX_CALLS: Field = 4;
// 1 (ARGS_HASH) + 1 (FUNCTION_SELECTOR) + 1 (TARGET_ADDRESS) + 1 (IS_PUBLIC)
global FUNCTION_CALL_SIZE: Field = 4;
// 3 * 32 + 1
global FUNCTION_CALL_SIZE_IN_BYTES: Field = 97;

struct FunctionCall {
  args_hash: Field,
  function_selector: Field,
  target_address: Field,
  is_public: bool,
}

impl FunctionCall {
  fn serialize(self) -> [Field; FUNCTION_CALL_SIZE] {
    [self.args_hash, self.function_selector, self.target_address, self.is_public as Field]
  }

  fn to_be_bytes(self) -> [u8; FUNCTION_CALL_SIZE_IN_BYTES] {
    let mut bytes: [u8; FUNCTION_CALL_SIZE_IN_BYTES] = [0; FUNCTION_CALL_SIZE_IN_BYTES];
    let args_hash_bytes = self.args_hash.to_be_bytes(32);
    for i in 0..32 { bytes[i] = args_hash_bytes[i]; }
    let function_selector_bytes = self.function_selector.to_be_bytes(32);
    for i in 0..32 { bytes[i + 32] = function_selector_bytes[i]; }
    let target_address_bytes = self.target_address.to_be_bytes(32);
    for i in 0..32 { bytes[i + 64] = target_address_bytes[i]; }
    bytes[96] = self.is_public as u8;
    bytes
  }
}

// FUNCTION_CALL_SIZE * ACCOUNT_MAX_CALLS + 1
global ENTRYPOINT_PAYLOAD_SIZE: Field = 17;
// FUNCTION_CALL_SIZE_IN_BYTES * ACCOUNT_MAX_CALLS + 32
global ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES: Field = 420;

// docs:start:entrypoint-struct
struct EntrypointPayload {
  function_calls: [FunctionCall; ACCOUNT_MAX_CALLS],
  nonce: Field,
}
// docs:end:entrypoint-struct

impl EntrypointPayload {
  fn hash(self) -> Field {
    hash::pedersen_with_separator(
      self.serialize(), 
      GENERATOR_INDEX__SIGNATURE_PAYLOAD
    )[0]
  }

  // Serializes the entrypoint struct
  fn serialize(self) -> [Field; ENTRYPOINT_PAYLOAD_SIZE] {
    let mut fields: BoundedVec<Field, ENTRYPOINT_PAYLOAD_SIZE> = BoundedVec::new(0); 
    for call in self.function_calls {
      fields.push_array(call.serialize());
    }
    fields.push(self.nonce);
    fields.storage
  }

  // Serializes the payload as an array of bytes. Useful for hashing with sha256.
  fn to_be_bytes(self) -> [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] {
    let mut bytes: [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = [0; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES];

    for i in 0..ACCOUNT_MAX_CALLS {
      let item_bytes = self.function_calls[i].to_be_bytes();
      for j in 0..FUNCTION_CALL_SIZE_IN_BYTES { 
        bytes[i * FUNCTION_CALL_SIZE_IN_BYTES + j] = item_bytes[j];
      }
    }

    let nonce_bytes = self.nonce.to_be_bytes(32);
    let nonce_offset = FUNCTION_CALL_SIZE_IN_BYTES * ACCOUNT_MAX_CALLS;
    for j in 0..32 {
      bytes[nonce_offset + j] = nonce_bytes[j];
    }
    
    bytes
  }

  // Executes all private and public calls 
  // docs:start:entrypoint-execute-calls
  fn execute_calls(self, context: &mut PrivateContext) {
    for call in self.function_calls {
      if call.target_address != 0 {
        if call.is_public {
          context.call_public_function_with_packed_args(
            call.target_address, call.function_selector, call.args_hash
          );
        } else {
          let _result = context.call_private_function_with_packed_args(
            call.target_address, call.function_selector, call.args_hash
          );
        }
      }
    }
  }
  // docs:end:entrypoint-execute-calls
}