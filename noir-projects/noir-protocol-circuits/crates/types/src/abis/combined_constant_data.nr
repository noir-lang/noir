use crate::transaction::tx_context::TxContext;
use crate::header::Header;
use crate::traits::{Empty, Serialize, Deserialize};
use crate::abis::global_variables::GlobalVariables;
use crate::constants::COMBINED_CONSTANT_DATA_LENGTH;
use crate::utils::reader::Reader;

struct CombinedConstantData {
    historical_header: Header,
    // Note: `chain_id` and `version` in tx_context are not redundant to the values in
    // self.historical_header.global_variables because they can be different in case of a protocol upgrade. In such
    // a situation we could be using header from a block before the upgrade took place but be using the updated
    // protocol to execute and prove the transaction.
    tx_context: TxContext,

    vk_tree_root: Field,

    // Must be empty in private, will be filled in by the public kernel
    global_variables: GlobalVariables,
}

impl CombinedConstantData {
    pub fn private(
        historical_header: Header,
        tx_context: TxContext,
        vk_tree_root: Field
    ) -> CombinedConstantData {
        CombinedConstantData { historical_header, tx_context, vk_tree_root, global_variables: GlobalVariables::empty() }
    }
}

impl Empty for CombinedConstantData {
    fn empty() -> Self {
        CombinedConstantData {
            historical_header: Header::empty(),
            tx_context: TxContext::empty(),
            vk_tree_root: 0,
            global_variables: GlobalVariables::empty()
        }
    }
}

impl Serialize<COMBINED_CONSTANT_DATA_LENGTH> for CombinedConstantData {
  fn serialize(self) -> [Field; COMBINED_CONSTANT_DATA_LENGTH] {
    let mut fields: BoundedVec<Field, COMBINED_CONSTANT_DATA_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.historical_header.serialize());
    fields.extend_from_array(self.tx_context.serialize());
    fields.push(self.vk_tree_root);
    fields.extend_from_array(self.global_variables.serialize());

    assert_eq(fields.len(), COMBINED_CONSTANT_DATA_LENGTH);

    fields.storage
  }
}

impl Deserialize<COMBINED_CONSTANT_DATA_LENGTH> for CombinedConstantData {
  fn deserialize(fields: [Field; COMBINED_CONSTANT_DATA_LENGTH]) -> CombinedConstantData {
    let mut reader = Reader::new(fields);

    let item = CombinedConstantData {
      historical_header: reader.read_struct(Header::deserialize),
      tx_context: reader.read_struct(TxContext::deserialize),
      vk_tree_root: reader.read(),
      global_variables: reader.read_struct(GlobalVariables::deserialize),
    };
    reader.finish();
    item
  }
}

impl Eq for CombinedConstantData {
  fn eq(self, other: Self) -> bool {
    (self.historical_header.eq(other.historical_header)) &
    (self.tx_context.eq(other.tx_context)) &
    (self.vk_tree_root.eq(other.vk_tree_root)) &
    (self.global_variables.eq(other.global_variables))
  }
}

#[test]
fn serialization_of_empty() {
    let item = CombinedConstantData::empty();
    let serialized = item.serialize();
    let deserialized = CombinedConstantData::deserialize(serialized);
    assert(item.eq(deserialized));
}
