use crate::abis::{call_context::CallContext, function_data::FunctionData, gas::Gas};
use crate::address::AztecAddress;
use crate::constants::{GENERATOR_INDEX__CALL_STACK_ITEM, PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH};
use crate::traits::{Hash, Empty, Serialize, Deserialize};
use crate::utils::reader::Reader;

/**
 * A compressed version of the PublicCallStackItem struct used to compute the "hash"
 * of a PublicCallStackItem.
 * 
 * Historically, we have been zeroing most values in the PublicCallStackItem struct
 * to compute the hash involved when adding a PublicCallStackItem to the PublicCallStack.
 * 
 * This struct is used to store the values that we did not zero out, and allow us to hash
 * only these, thereby skipping a lot of computation and saving us a lot of constraints
 * 
 * Essentially this struct exists such that we don't have a `hash` function in the 
 * PublicCallStackItem struct that practically throws away some values of the struct
 * without clearly indicating that it does so.
 */
struct PublicCallStackItemCompressed {
    contract_address: AztecAddress,
    call_context: CallContext,
    function_data: FunctionData,
    args_hash: Field,
    returns_hash: Field,
    revert_code: u8,
    start_gas_left: Gas,
    end_gas_left: Gas,
}

impl Eq for PublicCallStackItemCompressed {
    fn eq(self, other: PublicCallStackItemCompressed) -> bool {
        (self.contract_address == other.contract_address)
            & (self.call_context == other.call_context)
            & (self.function_data == other.function_data)
            & (self.args_hash == other.args_hash)
            & (self.returns_hash == other.returns_hash)
            & (self.revert_code == other.revert_code)
            & (self.start_gas_left == other.start_gas_left)
            & (self.end_gas_left == other.end_gas_left)
    }
}

impl Hash for PublicCallStackItemCompressed {
    fn hash(self) -> Field {
        std::hash::pedersen_hash_with_separator(self.serialize(), GENERATOR_INDEX__CALL_STACK_ITEM)
    }
}

impl Empty for PublicCallStackItemCompressed {
    fn empty() -> Self {
        PublicCallStackItemCompressed {
            contract_address: AztecAddress::empty(),
            call_context: CallContext::empty(),
            function_data: FunctionData::empty(),
            args_hash: 0,
            returns_hash: 0,
            revert_code: 0,
            start_gas_left: Gas::empty(),
            end_gas_left: Gas::empty(),
        }
    }
}

impl Serialize<PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH> for PublicCallStackItemCompressed {
    fn serialize(self) -> [Field; PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH> = BoundedVec::new();

        fields.push(self.contract_address.to_field());
        fields.extend_from_array(self.call_context.serialize());
        fields.extend_from_array(self.function_data.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);
        fields.push(self.revert_code as Field);
        fields.extend_from_array(self.start_gas_left.serialize());
        fields.extend_from_array(self.end_gas_left.serialize());

        assert_eq(fields.len(), PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH);

        fields.storage
    }
}

impl Deserialize<PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH> for PublicCallStackItemCompressed {
  fn deserialize(fields: [Field; PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH]) -> PublicCallStackItemCompressed {
    let mut reader = Reader::new(fields);

    let item = PublicCallStackItemCompressed {
      contract_address: reader.read_struct(AztecAddress::deserialize),
      call_context: reader.read_struct(CallContext::deserialize),
      function_data: reader.read_struct(FunctionData::deserialize),
      args_hash: reader.read(),
      returns_hash: reader.read(),
      revert_code: reader.read() as u8,
      start_gas_left: reader.read_struct(Gas::deserialize),
      end_gas_left: reader.read_struct(Gas::deserialize),
    };
    reader.finish();
    item
  }
}
