use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::Serialize};

// 1 (ARGS_HASH) + 1 (FUNCTION_SELECTOR) + 1 (TARGET_ADDRESS) + 1 (IS_PUBLIC) + 1 (IS_STATIC)
global FUNCTION_CALL_SIZE: Field = 5;
// 3 * 32 + 2
global FUNCTION_CALL_SIZE_IN_BYTES: Field = 98;

struct FunctionCall {
  args_hash: Field,
  function_selector: FunctionSelector,
  target_address: AztecAddress,
  is_public: bool,
  is_static: bool,
}

impl Serialize<FUNCTION_CALL_SIZE> for FunctionCall {
  fn serialize(self) -> [Field; FUNCTION_CALL_SIZE] {
    [self.args_hash, self.function_selector.to_field(), self.target_address.to_field(), self.is_public as Field, self.is_static as Field]
  }
}

impl FunctionCall {
    fn to_be_bytes(self) -> [u8; FUNCTION_CALL_SIZE_IN_BYTES] {
        let mut bytes: [u8; FUNCTION_CALL_SIZE_IN_BYTES] = [0; FUNCTION_CALL_SIZE_IN_BYTES];
        let args_hash_bytes = self.args_hash.to_be_bytes(32);
        for i in 0..32 {
            bytes[i] = args_hash_bytes[i];
        }
        let function_selector_bytes = self.function_selector.to_field().to_be_bytes(32);
        for i in 0..32 {
            bytes[i + 32] = function_selector_bytes[i];
        }
        let target_address_bytes = self.target_address.to_field().to_be_bytes(32);
        for i in 0..32 {
            bytes[i + 64] = target_address_bytes[i];
        }
        bytes[96] = self.is_public as u8;
        bytes[97] = self.is_static as u8;
        bytes
    }
}
