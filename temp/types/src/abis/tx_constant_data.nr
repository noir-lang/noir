use crate::{
    block_header::BlockHeader,
    constants::TX_CONSTANT_DATA_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    transaction::tx_context::TxContext,
    utils::reader::Reader,
};

// Constants used throughout the executions of both private and public.
pub struct TxConstantData {
    pub historical_header: BlockHeader,
    // Note: `chain_id` and `version` in tx_context are not redundant to the values in
    // self.historical_header.global_variables because they can be different in case of a protocol upgrade. In such
    // a situation we could be using header from a block before the upgrade took place but be using the updated
    // protocol to execute and prove the transaction.
    pub tx_context: TxContext,
    pub vk_tree_root: Field,
    pub protocol_contract_tree_root: Field,
}

impl Empty for TxConstantData {
    fn empty() -> Self {
        TxConstantData {
            historical_header: BlockHeader::empty(),
            tx_context: TxContext::empty(),
            vk_tree_root: 0,
            protocol_contract_tree_root: 0,
        }
    }
}

impl Eq for TxConstantData {
    fn eq(self, other: Self) -> bool {
        (self.historical_header.eq(other.historical_header))
            & (self.tx_context.eq(other.tx_context))
            & (self.vk_tree_root.eq(other.vk_tree_root))
            & (self.protocol_contract_tree_root.eq(other.protocol_contract_tree_root))
    }
}

impl Serialize<TX_CONSTANT_DATA_LENGTH> for TxConstantData {
    fn serialize(self) -> [Field; TX_CONSTANT_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, TX_CONSTANT_DATA_LENGTH> = BoundedVec::new();

        fields.extend_from_array(self.historical_header.serialize());
        fields.extend_from_array(self.tx_context.serialize());
        fields.push(self.vk_tree_root);
        fields.push(self.protocol_contract_tree_root);

        assert_eq(fields.len(), TX_CONSTANT_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<TX_CONSTANT_DATA_LENGTH> for TxConstantData {
    fn deserialize(fields: [Field; TX_CONSTANT_DATA_LENGTH]) -> TxConstantData {
        let mut reader = Reader::new(fields);

        let item = TxConstantData {
            historical_header: reader.read_struct(BlockHeader::deserialize),
            tx_context: reader.read_struct(TxContext::deserialize),
            vk_tree_root: reader.read(),
            protocol_contract_tree_root: reader.read(),
        };
        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty_tx_constant_data() {
    let item = TxConstantData::empty();
    let serialized = item.serialize();
    let deserialized = TxConstantData::deserialize(serialized);
    assert(item.eq(deserialized));
}
