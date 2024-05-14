use dep::types::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    constants::BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH, partial_state_reference::PartialStateReference,
    mocked::AggregationObject, traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};
use crate::abis::constant_rollup_data::ConstantRollupData;

global BASE_ROLLUP_TYPE = 0;
global MERGE_ROLLUP_TYPE = 1;

struct BaseOrMergeRollupPublicInputs {
    // rollup_type is either 0 (base) or 1 (merge)
    // TODO(Kev): Why is this a u32 instead of a u8/u16?
    rollup_type : u32,
    // subtree height is always 0 for base.
    // so that we always pass-in two base/merge circuits of the same height into the next level of recursion
    height_in_block_tree : Field,
    aggregation_object : AggregationObject,
    constants : ConstantRollupData,

    start: PartialStateReference,
    end: PartialStateReference,

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    // U128 isn't safe if it's an input to the circuit (it won't automatically constrain the witness)
    // So we want to constrain it when casting these fields to U128

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    txs_effects_hash : Field,
    out_hash : Field, 
}

impl Empty for BaseOrMergeRollupPublicInputs {
    fn empty() -> Self {
        BaseOrMergeRollupPublicInputs {
            rollup_type : 0 as u32,
            height_in_block_tree : 0,
            aggregation_object : AggregationObject::empty(),
            constants : ConstantRollupData::empty(),
            start: PartialStateReference::empty(),
            end: PartialStateReference::empty(),
            txs_effects_hash : 0,
            out_hash : 0, 
        }
    }
}

impl Eq for BaseOrMergeRollupPublicInputs {
  fn eq(self, other: Self) -> bool {
    (self.rollup_type == other.rollup_type) &
    (self.height_in_block_tree == other.height_in_block_tree) &
    (self.constants.eq(other.constants)) &
    (self.start.eq(other.start)) &
    (self.end.eq(other.end)) &
    (self.txs_effects_hash == other.txs_effects_hash) &
    (self.out_hash == other.out_hash)
  }
}

impl Serialize<BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> for BaseOrMergeRollupPublicInputs {
  fn serialize(self) -> [Field; BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH] {
    let mut fields: BoundedVec<Field, BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();

    fields.push(self.rollup_type as Field);
    fields.push(self.height_in_block_tree as Field);
    fields.extend_from_array(self.constants.serialize());
    fields.extend_from_array(self.start.serialize());
    fields.extend_from_array(self.end.serialize());
    fields.push(self.txs_effects_hash as Field);
    fields.push(self.out_hash as Field);

    assert_eq(fields.len(), BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> for BaseOrMergeRollupPublicInputs {
  fn deserialize(fields: [Field; BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH]) -> BaseOrMergeRollupPublicInputs {
    let mut reader = Reader::new(fields);
    let item = Self {
        rollup_type: reader.read() as u32,
        height_in_block_tree: reader.read(),
        aggregation_object : AggregationObject::empty(),
        constants: reader.read_struct(ConstantRollupData::deserialize),        
        start: reader.read_struct(PartialStateReference::deserialize),
        end: reader.read_struct(PartialStateReference::deserialize),
        txs_effects_hash: reader.read(),
        out_hash: reader.read(),
    };

    reader.finish();
    item
  }
}

#[test]
fn serialization_of_empty() {
    let item = BaseOrMergeRollupPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = BaseOrMergeRollupPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
