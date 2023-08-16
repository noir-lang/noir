use crate::abi;
use crate::types::vec::BoundedVec;
use crate::context::PrivateContext;
use crate::private_call_stack_item::PrivateCallStackItem;
use crate::public_call_stack_item::PublicCallStackItem;    

global ACCOUNT_MAX_PRIVATE_CALLS: Field = 2;
global ACCOUNT_MAX_PUBLIC_CALLS: Field = 2;
global ACCOUNT_MAX_CALLS: Field = 4;
// 1 (ARGS_HASH) + 1 (FUNCTION_SELECTOR) + 1 (TARGET_ADDRESS)
global FUNCTION_CALL_SIZE: Field = 3;

struct FunctionCall {
  args_hash: Field,
  function_selector: Field,
  target_address: Field,
}

impl FunctionCall {
  fn serialize(self) -> [Field; FUNCTION_CALL_SIZE] {
    [self.args_hash, self.function_selector, self.target_address]
  }
}

// FUNCTION_CALL_SIZE * (ACCOUNT_MAX_PUBLIC_CALLS + ACCOUNT_MAX_PRIVATE_CALLS) + 1
global ENTRYPOINT_PAYLOAD_SIZE: Field = 13;
global ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES: Field = 416;

struct EntrypointPayload {
  // Noir doesnt support nested arrays or structs yet so we flatten everything
  flattened_args_hashes: [Field; ACCOUNT_MAX_CALLS],
  flattened_selectors: [Field; ACCOUNT_MAX_CALLS],
  flattened_targets: [Field; ACCOUNT_MAX_CALLS],
  nonce: Field,
}

impl EntrypointPayload {
  // TODO(#1207) Do we need a generator index?
  fn hash(self) -> Field {
    dep::std::hash::pedersen(self.serialize())[0]
  }

  // Serializes the entrypoint struct
  fn serialize(self) -> [Field; ENTRYPOINT_PAYLOAD_SIZE] {
    let mut fields: BoundedVec<Field, ENTRYPOINT_PAYLOAD_SIZE> = BoundedVec::new(0); 
    fields.push_array(self.flattened_args_hashes);
    fields.push_array(self.flattened_selectors);
    fields.push_array(self.flattened_targets);
    fields.push(self.nonce);
    fields.storage
  }

  // Serializes the payload as an array of bytes. Useful for hashing with sha256.
  fn to_be_bytes(self) -> [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] {
    let mut bytes: [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = [0; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES];

    let args_len = self.flattened_args_hashes.len();
    let selectors_len = self.flattened_selectors.len();
    let targets_len = self.flattened_targets.len();

    for i in 0..args_len {
      let item_bytes = self.flattened_args_hashes[i].to_be_bytes(32);
      for j in 0..32 { 
        bytes[i * 32 + j] = item_bytes[j];
      }
    }

    for i in 0..selectors_len {
      let item_bytes = self.flattened_selectors[i].to_be_bytes(32);
      for j in 0..32 { 
        bytes[args_len * 32 + i * 32 + j] = item_bytes[j];
      }
    }

    for i in 0..targets_len {
      let item_bytes = self.flattened_targets[i].to_be_bytes(32);
      for j in 0..32 { 
        bytes[(args_len + selectors_len) * 32 + i * 32 + j] = item_bytes[j];
      }
    }
    
    let item_bytes = self.nonce.to_be_bytes(32);
    for j in 0..32 { 
      bytes[(args_len + selectors_len + targets_len) * 32 + j] = item_bytes[j];
    }

    bytes
  }

  // Executes all private and public calls 
  fn execute_calls(self, context: &mut PrivateContext) {
    for i in 0..ACCOUNT_MAX_PRIVATE_CALLS {
        let target_address = self.flattened_targets[i];
        if target_address != 0 {
            let function_selector = self.flattened_selectors[i];
            let args_hash = self.flattened_args_hashes[i];
            let _callStackItem = context.call_private_function_with_packed_args(target_address, function_selector, args_hash);
        }
    }
    for i in ACCOUNT_MAX_PRIVATE_CALLS..ACCOUNT_MAX_CALLS {
        let target_address = self.flattened_targets[i];
        if target_address != 0 {
            let function_selector = self.flattened_selectors[i];
            let args_hash = self.flattened_args_hashes[i];
            let _callStackItem = context.call_public_function_with_packed_args(target_address, function_selector, args_hash);
        }
    }
  }
}