use dep::aztec3::abi;
use dep::aztec3::types::vec::BoundedVec;

global ACCOUNT_MAX_PRIVATE_CALLS: comptime Field = 1;
global ACCOUNT_MAX_PUBLIC_CALLS: comptime Field = 1;
global ACCOUNT_MAX_CALLS: comptime Field = 2;

global FUNCTION_CALL_SIZE: comptime Field = 10;

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
global ENTRYPOINT_PAYLOAD_SIZE: comptime Field = 21;

// MAX_ARGS * (ACCOUNT_MAX_PUBLIC_CALLS + ACCOUNT_MAX_PRIVATE_CALLS)
global ACCOUNT_FLATTENED_ARGS_SIZE: comptime Field = 16;

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
}