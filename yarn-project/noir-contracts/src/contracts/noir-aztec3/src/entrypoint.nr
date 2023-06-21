use crate::abi;
use crate::types::vec::BoundedVec;

global ACCOUNT_MAX_PRIVATE_CALLS: comptime Field = 1;
global ACCOUNT_MAX_PUBLIC_CALLS: comptime Field = 1;
global ACCOUNT_MAX_CALLS: comptime Field = 2;
// MAX_ARGS + 1 (FUNCTION_SELECTOR) + 1 (TARGET_ADDRESS)
global FUNCTION_CALL_SIZE: comptime Field = 18;

struct FunctionCall {
  args: [Field; abi::MAX_ARGS],
  function_selector: Field,
  target_address: Field,
}

impl FunctionCall {
  fn serialize(self) -> [Field; FUNCTION_CALL_SIZE] {
    let mut fields: BoundedVec<Field, FUNCTION_CALL_SIZE> = BoundedVec::new(0); 
    fields = fields.push_array(self.args);
    fields = fields.push(self.function_selector);
    fields = fields.push(self.target_address);
    fields.storage
  }
}

// FUNCTION_CALL_SIZE * (ACCOUNT_MAX_PUBLIC_CALLS + ACCOUNT_MAX_PRIVATE_CALLS) + 1
global ENTRYPOINT_PAYLOAD_SIZE: comptime Field = 37;
global ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES: comptime Field = 1184;

// MAX_ARGS * (ACCOUNT_MAX_PUBLIC_CALLS + ACCOUNT_MAX_PRIVATE_CALLS)
global ACCOUNT_FLATTENED_ARGS_SIZE: comptime Field = 32;

struct EntrypointPayload {
  // Noir doesnt support nested arrays or structs yet so we flatten everything
  flattened_args: [Field; ACCOUNT_FLATTENED_ARGS_SIZE],
  flattened_selectors: [Field; ACCOUNT_MAX_CALLS],
  flattened_targets: [Field; ACCOUNT_MAX_CALLS],
  nonce: Field,
}

impl EntrypointPayload {
  // TODO hash_index
  fn hash(self) -> Field {
    dep::std::hash::pedersen(self.serialize())[0]
  }

  // Serializes the entrypoint struct
  fn serialize(self) -> [Field; ENTRYPOINT_PAYLOAD_SIZE] {
    let mut fields: BoundedVec<Field, ENTRYPOINT_PAYLOAD_SIZE> = BoundedVec::new(0); 
    fields = fields.push_array(self.flattened_args);
    fields = fields.push_array(self.flattened_selectors);
    fields = fields.push_array(self.flattened_targets);
    fields = fields.push(self.nonce);
    fields.storage
  }

  // Serializes the payload as an array of bytes. Useful for hashing with sha256.
  fn to_be_bytes(self) -> [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] {
    let mut bytes: [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = [0; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES];

    let args_len = self.flattened_args.len();
    let selectors_len = self.flattened_selectors.len();
    let targets_len = self.flattened_targets.len();

    for i in 0..args_len {
      let item_bytes = self.flattened_args[i].to_be_bytes(32);
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

  // TODO: This function returns all zeroes. Figure out why!
  fn to_be_bytes_fail(self) -> [u8; ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] {
    let mut bytes: BoundedVec<u8, ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES> = BoundedVec::new(0);

    // Noir crashes if we try to push_bytes the whole item_bytes into the vec
    // See https://github.com/noir-lang/noir/issues/1023
    for item in self.flattened_args {
      let item_bytes = item.to_be_bytes(32);
      for i in 0..32 { 
        bytes.push(item_bytes[i]); 
      }
    }
    for item in self.flattened_selectors {
      let item_bytes = item.to_be_bytes(32);
      for i in 0..32 { 
        bytes.push(item_bytes[i]); 
      }
    }
    for item in self.flattened_targets {
      let item_bytes = item.to_be_bytes(32);
      for i in 0..32 { 
        bytes.push(item_bytes[i]); 
      }
    }
    
    let nonce_bytes = self.nonce.to_be_bytes(32);
    for i in 0..32 { 
      bytes.push(nonce_bytes[i]); 
    }
    bytes.storage
  }
}